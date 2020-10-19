mod x11_api;

use crate::{PlatformApi, Result};
use x11_api::X11Api;

pub fn setup() -> Result<Box<dyn PlatformApi>> {
    Ok(Box::new(X11Api::new()?))
}
