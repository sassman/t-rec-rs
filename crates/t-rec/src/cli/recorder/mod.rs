//! Recording session orchestration, thread lifecycle, and visual feedback.

mod presenter;
pub mod runtime;
mod session;

pub use session::{PostProcessConfig, RecordingResult, RecordingSession, SessionConfig};
