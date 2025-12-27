//! Screenshot capture during recording.

use std::path::PathBuf;

/// Information about a captured screenshot.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScreenshotInfo {
    /// Timecode in milliseconds from recording start.
    pub timecode_ms: u128,
    /// Path to the screenshot file in tempdir.
    pub temp_path: PathBuf,
}

/// Generate a screenshot filename.
pub fn screenshot_file_name(timecode: u128, ext: &str) -> String {
    format!("t-rec-screenshot-{:09}.{}", timecode, ext)
}

/// Generate the final output filename for a screenshot.
pub fn screenshot_output_name(base: &str, timecode: u128, format: &str) -> String {
    format!("{}_screenshot-{}.{}", base, timecode, format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_file_name() {
        assert_eq!(
            screenshot_file_name(12345, "bmp"),
            "t-rec-screenshot-000012345.bmp"
        );
    }

    #[test]
    fn test_screenshot_output_name() {
        assert_eq!(
            screenshot_output_name("t-rec", 12345, "png"),
            "t-rec_screenshot-12345.png"
        );
    }
}
