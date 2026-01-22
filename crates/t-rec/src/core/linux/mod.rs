mod x11_api;

use crate::core::common::{Platform, PlatformApi, PlatformApiFactory};
use crate::core::Result;
use x11_api::X11Api;

impl PlatformApiFactory for Platform {
    fn setup() -> Result<Box<dyn PlatformApi>> {
        Ok(Box::new(X11Api::new()?))
    }
}

// Used in binary crate only (main.rs)
pub const DEFAULT_SHELL: &str = "/bin/sh";
