mod x11_api;

use super::super::common::{Platform, PlatformApiFactory};
use super::common::{PlatformApi, Result};
use x11_api::X11Api;

impl PlatformApiFactory for Platform {
    fn setup() -> Result<Box<dyn PlatformApi>> {
        Ok(Box::new(X11Api::new()?))
    }
}

// Used in binary crate only (main.rs)
pub const DEFAULT_SHELL: &str = "/bin/sh";
