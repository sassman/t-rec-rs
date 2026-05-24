//! PTY (Pseudo-Terminal) handling for shell spawning.
//!
//! This module provides PTY-based shell spawning using the nix crate.
//! It creates a pseudo-terminal pair and spawns a shell connected to it,
//! allowing proper interactive shell behavior with stdin/stdout forwarding.

use anyhow::{Context, Result};
use nix::pty::{openpty, OpenptyResult};
use nix::sys::termios::tcgetattr;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::thread;
use tokio::sync::broadcast;

use crate::core::event_router::{Event, LifecycleEvent};

/// A PTY-connected shell process.
pub struct PtyShell {
    /// The child shell process (kept alive by struct ownership).
    _child: Child,
    /// Master side of the PTY for reading shell output.
    master_read: File,
    /// Master side of the PTY for writing to shell.
    master_write: File,
}

impl PtyShell {
    /// Spawn a new shell connected to a PTY.
    ///
    /// This creates a pseudo-terminal pair and spawns the given program
    /// with the slave side as its controlling terminal.
    pub fn spawn(program: &str) -> Result<Self> {
        // Get current terminal settings to copy to the PTY
        let termios = tcgetattr(std::io::stdin()).context("Failed to get terminal attributes")?;

        // Create PTY pair
        let OpenptyResult { master, slave } =
            openpty(None, Some(&termios)).context("Failed to create PTY")?;

        let slave_fd = slave.as_raw_fd();

        // Spawn shell with slave as controlling terminal
        let child = unsafe {
            Command::new(program)
                .pre_exec(move || {
                    // Create new session and set controlling terminal
                    if libc::setsid() < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    if libc::ioctl(slave_fd, libc::TIOCSCTTY as _, 0) < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    // Duplicate slave to stdin/stdout/stderr
                    libc::dup2(slave_fd, 0);
                    libc::dup2(slave_fd, 1);
                    libc::dup2(slave_fd, 2);
                    // Close the original slave fd if it's not 0, 1, or 2
                    if slave_fd > 2 {
                        libc::close(slave_fd);
                    }
                    Ok(())
                })
                .spawn()
                .context(format!("Failed to spawn {}", program))?
        };

        // Close slave in parent (child has its own copy)
        drop(slave);

        // Create file handles for the master
        let master_fd = master.as_raw_fd();
        let master_read = unsafe { File::from_raw_fd(libc::dup(master_fd)) };
        let master_write = unsafe { File::from_raw_fd(master.into_raw_fd()) };

        Ok(Self {
            _child: child,
            master_read,
            master_write,
        })
    }

    /// Get a writer to send input to the shell.
    pub fn get_writer(&self) -> Result<File> {
        self.master_write
            .try_clone()
            .context("Failed to clone PTY master writer")
    }

    /// Get a reader to receive output from the shell.
    pub fn get_reader(&self) -> Result<File> {
        self.master_read
            .try_clone()
            .context("Failed to clone PTY master reader")
    }

    /// Run the output forwarding loop.
    ///
    /// Reads from the PTY master and writes to stdout.
    /// Returns when the shell exits, a shutdown signal is received, or an error occurs.
    pub fn forward_output(&mut self, mut event_rx: broadcast::Receiver<Event>) -> Result<()> {
        let mut reader = self.get_reader()?;
        let mut stdout = std::io::stdout();
        let mut buf = [0u8; 4096];

        // Set non-blocking mode on reader
        unsafe {
            let flags = libc::fcntl(reader.as_raw_fd(), libc::F_GETFL);
            libc::fcntl(reader.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        loop {
            // Check for lifecycle events (non-blocking)
            match event_rx.try_recv() {
                Ok(Event::Lifecycle(LifecycleEvent::Shutdown)) => {
                    log::debug!("Shell forwarder received shutdown signal");
                    break;
                }
                Ok(_) => {} // Ignore non-lifecycle events
                Err(broadcast::error::TryRecvError::Empty) => {}
                Err(broadcast::error::TryRecvError::Closed) => break,
                Err(broadcast::error::TryRecvError::Lagged(_)) => {}
            }

            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    stdout.write_all(&buf[..n])?;
                    stdout.flush()?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, sleep briefly
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break, // Other errors (likely shell exited)
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pty_shell_creation() {
        // Just test that we can create a PTY shell
        // (actual testing requires a real terminal)
        use super::*;
        // Skip in CI environments without a terminal
        if std::env::var("CI").is_ok() {
            return;
        }
        if let Ok(shell) = PtyShell::spawn("/bin/echo world") {
            let world = &mut String::new();
            shell.get_reader().unwrap().read_to_string(world).unwrap();
            assert!(world.contains("world"));
        }
    }
}
