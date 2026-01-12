//! PTY (Pseudo-Terminal) handling for Windows using ConPTY.
//!
//! This module provides ConPTY-based shell spawning for Windows.
//! It creates a pseudo-console and spawns a shell connected to it,
//! allowing proper interactive shell behavior with stdin/stdout forwarding.

use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::mem::{size_of, zeroed};
use std::ptr::null_mut;
use std::thread;
use tokio::sync::broadcast;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows::Win32::System::Console::{
    ClosePseudoConsole, CreatePseudoConsole, COORD, HPCON, PSEUDOCONSOLE_INHERIT_CURSOR,
};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    CreateProcessW, InitializeProcThreadAttributeList, UpdateProcThreadAttribute,
    WaitForSingleObject, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, STARTUPINFOEXW,
};

use crate::event_router::{Event, LifecycleEvent};

/// A ConPTY-connected shell process for Windows.
pub struct PtyShell {
    /// Handle to the pseudo-console.
    hpc: HPCON,
    /// Process information for the spawned shell.
    process_info: PROCESS_INFORMATION,
    /// Pipe for reading from the pseudo-console (shell output).
    pipe_pty_out: HANDLE,
    /// Pipe for writing to the pseudo-console (shell input).
    pipe_pty_in: HANDLE,
    /// Attribute list (must be kept alive).
    _attr_list: Vec<u8>,
}

// SAFETY: The handles are owned by PtyShell and properly closed on drop.
// Windows handles can be used from any thread.
unsafe impl Send for PtyShell {}

impl Drop for PtyShell {
    fn drop(&mut self) {
        unsafe {
            // Close pipe handles first to signal EOF to the process
            let _ = CloseHandle(self.pipe_pty_in);
            let _ = CloseHandle(self.pipe_pty_out);

            // Close the pseudo-console (this should signal the process to exit)
            ClosePseudoConsole(self.hpc);

            // Brief wait for process to exit gracefully (100ms)
            WaitForSingleObject(self.process_info.hProcess, 100);

            // Close process handles
            let _ = CloseHandle(self.process_info.hProcess);
            let _ = CloseHandle(self.process_info.hThread);
        }
    }
}

impl PtyShell {
    /// Spawn a new shell connected to a ConPTY.
    ///
    /// This creates a pseudo-console and spawns the given program
    /// with the console as its controlling terminal.
    pub fn spawn(program: &str) -> Result<Self> {
        unsafe { Self::spawn_impl(program) }
    }

    unsafe fn spawn_impl(program: &str) -> Result<Self> {
        // Create pipes for PTY communication
        // pipe_in: we write to pipe_in_write, PTY reads from pipe_in_read
        // pipe_out: PTY writes to pipe_out_write, we read from pipe_out_read
        let mut pipe_in_read = INVALID_HANDLE_VALUE;
        let mut pipe_in_write = INVALID_HANDLE_VALUE;
        let mut pipe_out_read = INVALID_HANDLE_VALUE;
        let mut pipe_out_write = INVALID_HANDLE_VALUE;

        let sa = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            bInheritHandle: true.into(),
            lpSecurityDescriptor: null_mut(),
        };

        CreatePipe(&mut pipe_in_read, &mut pipe_in_write, Some(&sa), 0)
            .context("Failed to create input pipe")?;

        CreatePipe(&mut pipe_out_read, &mut pipe_out_write, Some(&sa), 0)
            .context("Failed to create output pipe")?;

        // Get console size (default to 120x30 if we can't determine)
        let size = COORD { X: 120, Y: 30 };

        // Create the pseudo-console
        let hpc = CreatePseudoConsole(
            size,
            pipe_in_read,   // PTY reads input from this pipe
            pipe_out_write, // PTY writes output to this pipe
            PSEUDOCONSOLE_INHERIT_CURSOR,
        )
        .context("Failed to create pseudo-console")?;

        // Close the handles that the pseudo-console now owns
        let _ = CloseHandle(pipe_in_read);
        let _ = CloseHandle(pipe_out_write);

        // Prepare startup info with pseudo-console attribute
        let mut attr_list_size: usize = 0;

        // First call to get required size (expected to fail, just gets size)
        let _ = InitializeProcThreadAttributeList(None, 1, None, &mut attr_list_size);

        // Allocate the attribute list
        let mut attr_list: Vec<u8> = vec![0; attr_list_size];
        let attr_list_ptr = LPPROC_THREAD_ATTRIBUTE_LIST(attr_list.as_mut_ptr() as *mut _);

        InitializeProcThreadAttributeList(Some(attr_list_ptr), 1, None, &mut attr_list_size)
            .context("Failed to initialize proc thread attribute list")?;

        // Add pseudo-console attribute
        UpdateProcThreadAttribute(
            attr_list_ptr,
            0,
            PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
            Some(hpc.0 as *const std::ffi::c_void),
            size_of::<HPCON>(),
            None,
            None,
        )
        .context("Failed to update proc thread attribute")?;

        // Create startup info
        let mut startup_info: STARTUPINFOEXW = zeroed();
        startup_info.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup_info.lpAttributeList = attr_list_ptr;

        // Create process
        let mut process_info: PROCESS_INFORMATION = zeroed();

        // Create the command line - spawn the program directly
        let mut cmd_wide: Vec<u16> = program.encode_utf16().chain(std::iter::once(0)).collect();

        // IMPORTANT: Cast &STARTUPINFOEXW to *const STARTUPINFOW (not &startup_info.StartupInfo)
        // This ensures the attribute list remains accessible
        CreateProcessW(
            None,
            Some(PWSTR(cmd_wide.as_mut_ptr())),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT,
            None,
            None,
            &startup_info.StartupInfo,
            &mut process_info,
        )
        .context(format!("Failed to create process: {}", program))?;

        Ok(Self {
            hpc,
            process_info,
            pipe_pty_out: pipe_out_read,
            pipe_pty_in: pipe_in_write,
            _attr_list: attr_list,
        })
    }

    /// Get a writer to send input to the shell.
    pub fn get_writer(&self) -> Result<PtyWriter> {
        Ok(PtyWriter {
            handle: self.pipe_pty_in,
        })
    }

    /// Get a reader to receive output from the shell.
    pub fn get_reader(&self) -> Result<PtyReader> {
        Ok(PtyReader {
            handle: self.pipe_pty_out,
        })
    }

    /// Run the output forwarding loop.
    ///
    /// Reads from the PTY and writes to stdout.
    /// Returns when the shell exits, a shutdown signal is received, or an error occurs.
    pub fn forward_output(&mut self, mut event_rx: broadcast::Receiver<Event>) -> Result<()> {
        let reader = self.get_reader()?;
        let mut stdout = std::io::stdout();
        let mut buf = [0u8; 4096];

        loop {
            // Check for lifecycle events (non-blocking)
            match event_rx.try_recv() {
                Ok(Event::Lifecycle(LifecycleEvent::Shutdown)) => {
                    log::debug!("Shell forwarder received shutdown signal");
                    break;
                }
                Ok(Event::Lifecycle(LifecycleEvent::Error(e))) => {
                    log::error!("Shell forwarder shutdown due to error: {}", e);
                    break;
                }
                Ok(_) => {} // Ignore non-lifecycle events
                Err(broadcast::error::TryRecvError::Empty) => {}
                Err(broadcast::error::TryRecvError::Closed) => break,
                Err(broadcast::error::TryRecvError::Lagged(_)) => {}
            }

            // Try to read from PTY (non-blocking via PeekNamedPipe would be better,
            // but for simplicity we'll use a small timeout approach)
            match reader.read_timeout(&mut buf, 10) {
                Ok(0) => {
                    // Check if process has exited
                    unsafe {
                        if WaitForSingleObject(self.process_info.hProcess, 0).0 == 0 {
                            break; // Process exited
                        }
                    }
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(n) => {
                    stdout.write_all(&buf[..n])?;
                    stdout.flush()?;
                }
                Err(_) => {
                    // Check if process has exited
                    unsafe {
                        if WaitForSingleObject(self.process_info.hProcess, 0).0 == 0 {
                            break; // Process exited
                        }
                    }
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }

        Ok(())
    }
}

/// Writer for sending input to the PTY.
pub struct PtyWriter {
    handle: HANDLE,
}

// SAFETY: Windows handles can be used from any thread
unsafe impl Send for PtyWriter {}
unsafe impl Sync for PtyWriter {}

impl Write for PtyWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut bytes_written: u32 = 0;
        unsafe {
            WriteFile(self.handle, Some(buf), Some(&mut bytes_written), None).map_err(
                |e: windows::core::Error| {
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                },
            )?;
        }
        Ok(bytes_written as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Reader for receiving output from the PTY.
pub struct PtyReader {
    handle: HANDLE,
}

// SAFETY: Windows handles can be used from any thread
unsafe impl Send for PtyReader {}
unsafe impl Sync for PtyReader {}

impl PtyReader {
    /// Read with a timeout in milliseconds.
    fn read_timeout(&self, buf: &mut [u8], _timeout_ms: u32) -> std::io::Result<usize> {
        use windows::Win32::System::Pipes::PeekNamedPipe;

        // Check if there's data available
        let mut bytes_available: u32 = 0;
        unsafe {
            if PeekNamedPipe(
                self.handle,
                None,
                0,
                None,
                Some(&mut bytes_available),
                None,
            )
            .is_err()
            {
                return Ok(0);
            }
        }

        if bytes_available == 0 {
            return Ok(0);
        }

        // Read available data
        let mut bytes_read: u32 = 0;
        let to_read = std::cmp::min(buf.len() as u32, bytes_available);
        unsafe {
            ReadFile(
                self.handle,
                Some(&mut buf[..to_read as usize]),
                Some(&mut bytes_read),
                None,
            )
            .map_err(|e: windows::core::Error| {
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })?;
        }

        Ok(bytes_read as usize)
    }
}

impl Read for PtyReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_timeout(buf, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_shell_spawn() {
        // Test that we can create a PTY shell with cmd.exe
        // This verifies the ConPTY API calls succeed
        let shell = PtyShell::spawn("cmd.exe");
        assert!(
            shell.is_ok(),
            "Failed to spawn PTY shell: {:?}",
            shell.err()
        );

        // Verify we can get reader and writer handles
        let shell = shell.unwrap();
        assert!(shell.get_reader().is_ok(), "Failed to get reader");
        assert!(shell.get_writer().is_ok(), "Failed to get writer");

        // Drop the shell (tests cleanup)
        drop(shell);
    }
}
