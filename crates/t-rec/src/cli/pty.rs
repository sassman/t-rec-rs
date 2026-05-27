//! PTY (Pseudo-Terminal) handling for shell spawning.
//!
//! This module provides PTY-based shell spawning using the nix crate.
//! It creates a pseudo-terminal pair and spawns a shell connected to it,
//! allowing proper interactive shell behavior with stdin/stdout forwarding.

use anyhow::{Context, Result};
use nix::pty::{openpty, OpenptyResult, Winsize};
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use tokio::sync::broadcast;

use crate::core::event_router::{CaptureEvent, Event, EventRouter, LifecycleEvent};

/// Set the FD_CLOEXEC flag so the descriptor closes across `execve`.
fn set_cloexec(fd: RawFd) -> std::io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(std::io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Query the controlling terminal for its current geometry.
///
/// Falls back to 80x24 if stdin isn't a TTY (the slave must have a non-zero size
/// or TUIs render to nothing).
fn parent_winsize() -> Winsize {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let ok = unsafe { libc::ioctl(libc::STDIN_FILENO, libc::TIOCGWINSZ, &mut ws) } == 0;

    if ok && ws.ws_row > 0 && ws.ws_col > 0 {
        Winsize {
            ws_row: ws.ws_row,
            ws_col: ws.ws_col,
            ws_xpixel: ws.ws_xpixel,
            ws_ypixel: ws.ws_ypixel,
        }
    } else {
        Winsize {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}

/// Handle for resizing the PTY slave from another thread.
///
/// Owns its own duplicate of the master fd so it can outlive the borrow on
/// `PtyShell` and be moved into an actor closure.
pub struct PtyResizer {
    master: File,
}

impl PtyResizer {
    /// Apply a new geometry to the slave. The kernel will deliver SIGWINCH
    /// to the slave's foreground process group, which is what triggers TUI
    /// redraws.
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        let ws = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let ret = unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ, &ws) };
        if ret != 0 {
            return Err(anyhow::Error::new(std::io::Error::last_os_error())
                .context("ioctl(TIOCSWINSZ) failed"));
        }
        Ok(())
    }
}

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
        // Inherit the parent terminal's geometry. Without this, the slave defaults to
        // 0x0 and TUI apps (gitui, vim, htop, …) render to an empty canvas — the visible
        // terminal stays black. See issue #346.
        let winsize = parent_winsize();

        // Pass None for termios: let the kernel apply its default line discipline
        // (cooked, echo on). Copying parent termios is fragile — if spawn ever happens
        // after we've enabled raw mode, the slave inherits raw and breaks the child's
        // line editing. Mirrors wezterm's approach.
        let OpenptyResult { master, slave } =
            openpty(&winsize, None).context("Failed to create PTY")?;

        // Make sure the child closes the master across exec. Without CLOEXEC the
        // child holds a reference to the master end, which means our reader on the
        // parent side never sees EOF when the child exits.
        set_cloexec(master.as_raw_fd()).context("Failed to set FD_CLOEXEC on PTY master")?;

        let slave_fd = slave.as_raw_fd();

        // Spawn shell with slave as controlling terminal
        let child = unsafe {
            Command::new(program)
                .pre_exec(move || {
                    // Reset signal dispositions. The Rust runtime sets SIGPIPE=SIG_IGN
                    // (so `println!` to a broken pipe doesn't kill us), and SIG_IGN is
                    // preserved across exec. Shells and most CLI tools expect default
                    // handling for these signals; inheriting SIG_IGN causes subtle bugs.
                    for &signo in &[
                        libc::SIGCHLD,
                        libc::SIGHUP,
                        libc::SIGINT,
                        libc::SIGQUIT,
                        libc::SIGTERM,
                        libc::SIGALRM,
                        libc::SIGPIPE,
                    ] {
                        libc::signal(signo, libc::SIG_DFL);
                    }
                    // The signal mask is inherited across exec (POSIX), so reset it.
                    let empty: libc::sigset_t = std::mem::zeroed();
                    libc::sigprocmask(libc::SIG_SETMASK, &empty, std::ptr::null_mut());

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

    /// Get a handle that can resize the slave from another thread.
    pub fn get_resizer(&self) -> Result<PtyResizer> {
        let master = self
            .master_write
            .try_clone()
            .context("Failed to clone PTY master for resize")?;
        Ok(PtyResizer { master })
    }

    /// Run the output forwarding loop.
    ///
    /// Reads from the PTY master and writes to stdout. When the shell exits
    /// (naturally via `exit` or because it crashed), broadcasts `CaptureEvent::Stop`
    /// so the session unwinds — same lifecycle path as if the user had pressed Ctrl+D.
    pub fn forward_output(
        &mut self,
        mut event_rx: broadcast::Receiver<Event>,
        router: EventRouter,
    ) -> Result<()> {
        let mut reader = self.get_reader()?;
        let mut stdout = std::io::stdout();
        let mut buf = [0u8; 4096];
        let reader_fd = reader.as_raw_fd();

        let mut shutdown_received = false;

        // POLL_TIMEOUT_MS caps how long we sit in poll() without checking
        // lifecycle events or the child's status. 100ms gives near-instant
        // shutdown response without the cost of a busy loop.
        const POLL_TIMEOUT_MS: libc::c_int = 100;

        loop {
            // Check for lifecycle events (non-blocking)
            match event_rx.try_recv() {
                Ok(Event::Lifecycle(LifecycleEvent::Shutdown)) => {
                    log::debug!("Shell forwarder received shutdown signal");
                    shutdown_received = true;
                    break;
                }
                Ok(_) => {} // Ignore non-lifecycle events
                Err(broadcast::error::TryRecvError::Empty) => {}
                Err(broadcast::error::TryRecvError::Closed) => break,
                Err(broadcast::error::TryRecvError::Lagged(_)) => {}
            }

            // On macOS, a non-blocking read from a PTY master whose slave has been
            // closed can return EAGAIN indefinitely instead of EOF/EIO. Polling the
            // child's exit status is the reliable way to notice the shell is gone.
            if matches!(self._child.try_wait(), Ok(Some(_))) {
                log::debug!("Shell process exited; stopping forwarder");
                break;
            }

            // Block in the kernel until master has data, hangs up, or our timeout
            // elapses. Avoids the 10ms latency floor and 0% CPU spin of the old
            // non-blocking+sleep loop — TUI redraws now reach stdout as soon as
            // the shell produces them.
            let mut pfd = libc::pollfd {
                fd: reader_fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let n = unsafe { libc::poll(&mut pfd, 1, POLL_TIMEOUT_MS) };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                log::warn!("poll() on PTY master failed: {}", err);
                break;
            }
            if n == 0 {
                // Timeout — loop to recheck lifecycle and child status.
                continue;
            }

            // poll reported the fd is readable or hung up; let read() decide which.
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF (slave closed)
                Ok(n) => {
                    stdout.write_all(&buf[..n])?;
                    stdout.flush()?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break, // EIO / other terminal errors
            }
        }

        // If we exited because the shell died (not because shutdown was already
        // initiated), tell the session to stop. Mirrors the Ctrl+D path.
        if !shutdown_received {
            router.send(Event::Capture(CaptureEvent::Stop));
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
