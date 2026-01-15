mod x11_api;

use crate::common::{Platform, PlatformApiFactory};
use crate::{PlatformApi, Result};
use x11_api::X11Api;

impl PlatformApiFactory for Platform {
    fn setup() -> Result<Box<dyn PlatformApi>> {
        Ok(Box::new(X11Api::new()?))
    }
}

#[allow(dead_code)]
pub fn setup() -> Result<impl PlatformApi> {
    X11Api::new()
}

#[allow(dead_code)]
pub const DEFAULT_SHELL: &str = "/bin/sh";
