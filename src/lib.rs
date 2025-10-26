pub mod capture;
pub mod cli;
pub mod config;
pub mod error;
pub mod export;
pub mod processing;

#[cfg(feature = "gui")]
pub mod ui;

#[cfg(feature = "gui")]
pub mod animation;

#[cfg(feature = "gui")]
pub mod tools;

use async_trait::async_trait;
use serde::{Deserialize, Serialize}
pub use error::{Result, ScreenshotError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptureMode {
    Screen,
    Window,
    Region,
    Monitor,
}
