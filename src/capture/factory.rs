use crate::{Result, ScreenshotBackend};

pub fn create_backend() -> Result<Box<dyn ScreenshotBackend>> {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Ok(Box::new(crate::capture::WaylandBackend::new()?))
    } else {
        Err(crate::error::ScreenshotError::BackendUnavailable)
    }
}
