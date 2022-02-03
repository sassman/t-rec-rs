mod x11_api;

use crate::{PlatformApi, Result};
use x11_api::X11Api;

pub fn setup() -> Result<impl PlatformApi> {
    Ok(X11Api::new()?)
}

pub const DEFAULT_SHELL: &str = "/bin/sh";
