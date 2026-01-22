//! CLI-specific modules for the t-rec binary.

pub mod args;
pub mod config;
pub mod input;
pub mod logging;
pub mod output;
pub mod prompt;
pub mod recorder;
pub mod summary;
pub mod tips;
pub mod utils;

#[cfg(unix)]
pub mod pty;

// Re-export commonly used items
pub use args::{launch, resolve_profiled_settings, CliArgs};
pub use config::{expand_home, handle_init_config, handle_list_profiles, ProfileSettings};
pub use logging::init_logging;
pub use output::OutputGenerator;
pub use recorder::{RecordingSession, SessionConfig};
pub use summary::print_recording_summary;
