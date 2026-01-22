use dialoguer::console::Term;
use dialoguer::Confirm;
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Result of a timed prompt
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PromptResult {
    Yes,
    No,
    Timeout,
}

/// Handle for a background prompt that can be awaited later.
pub struct BackgroundPrompt {
    handle: JoinHandle<PromptResult>,
}

impl BackgroundPrompt {
    /// Wait for the prompt to complete and return the result.
    pub fn wait(self) -> PromptResult {
        self.handle.join().unwrap_or(PromptResult::No)
    }
}

/// Starts an interactive yes/no prompt in the background.
///
/// The prompt runs in a separate thread, allowing other work to proceed
/// while waiting for user input. Call `.wait()` on the returned handle
/// to get the result.
///
/// Returns `None` if stdin is not interactive (piped/redirected).
pub fn start_background_prompt(question: &str, timeout_secs: u64) -> Option<BackgroundPrompt> {
    if !is_interactive() {
        return None;
    }

    let question = question.to_string();
    let handle = thread::spawn(move || run_prompt(&question, timeout_secs));

    Some(BackgroundPrompt { handle })
}

/// Runs the interactive prompt with countdown.
fn run_prompt(question: &str, timeout_secs: u64) -> PromptResult {
    let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    // Spawn thread to run dialoguer prompt
    let question_clone = question.to_string();
    thread::spawn(move || {
        let result = Confirm::new()
            .with_prompt(&question_clone)
            .default(false)
            .interact();

        if let Ok(confirmed) = result {
            let _ = tx.send(confirmed);
        }
    });

    // Small delay to let dialoguer render its prompt first
    thread::sleep(Duration::from_millis(50));

    // Countdown loop - use carriage return to update in place on same line
    for remaining in (0..=timeout_secs).rev() {
        // Save cursor, move to column 0 of next line, print countdown, restore cursor
        // This prints below dialoguer without interfering with it
        print!("\x1b[s\n\r(auto-skip in {}s)  \x1b[u", remaining);
        io::stdout().flush().unwrap();

        if remaining == 0 {
            break;
        }

        // Check for input with 1-second timeout
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(confirmed) => {
                // Clear the countdown line (move down, clear, move back up)
                print!("\n\r\x1b[2K\x1b[1A");
                io::stdout().flush().unwrap();
                return if confirmed {
                    PromptResult::Yes
                } else {
                    println!("\n\nSkipping video generation");
                    PromptResult::No
                };
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Continue countdown
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                print!("\n\r\x1b[2K\x1b[1A");
                io::stdout().flush().unwrap();
                return PromptResult::No;
            }
        }
    }

    // Restore terminal state before returning
    restore_terminal();
    println!("\n\nSkipping video generation (timeout)");
    PromptResult::Timeout
}

/// Restore terminal to normal state.
///
/// This ensures the cursor is visible and terminal modes are reset
/// after dialoguer's prompt, especially important when timeout occurs
/// and the prompt thread is abandoned.
fn restore_terminal() {
    let term = Term::stdout();
    let _ = term.show_cursor();
    // Clear any remaining input state by flushing
    let _ = io::stdout().flush();
}

/// Check if stdin is connected to an interactive terminal.
///
/// Returns false if input is piped or redirected.
fn is_interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_result_enum() {
        assert_ne!(PromptResult::Yes, PromptResult::No);
        assert_ne!(PromptResult::No, PromptResult::Timeout);
        assert_ne!(PromptResult::Yes, PromptResult::Timeout);
    }
}
