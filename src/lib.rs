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
use serde::{Deserialize, Serialize};

pub use error::{Result, ScreenshotError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptureMode {
    Screen,
    Window,
    Region,
    Monitor,
}

impl std::str::FromStr for CaptureMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "screen" | "fullscreen" | "full" => Ok(CaptureMode::Screen),
            "window" | "win" => Ok(CaptureMode::Window),
            "region" | "area" | "selection" | "select" => Ok(CaptureMode::Region),
            "monitor" | "display" => Ok(CaptureMode::Monitor),
            _ => Err(format!("Invalid capture mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Png,
    Jpeg,
    Webp,
    Clipboard,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "png" => Ok(OutputFormat::Png),
            "jpg" | "jpeg" => Ok(OutputFormat::Jpeg),
            "webp" => Ok(OutputFormat::Webp),
            "clip" | "clipboard" => Ok(OutputFormat::Clipboard),
            _ => Err(format!("Invalid format: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotMetadata {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub mode: CaptureMode,
    pub width: u32,
    pub height: u32,
    pub format: OutputFormat,
    pub file_size: Option<u64>,
}

#[async_trait]
pub trait ScreenshotBackend: Send + Sync {
    async fn capture(&self, mode: CaptureMode, options: &CaptureOptions) -> Result<Screenshot>;

    async fn get_displays(&self) -> Result<Vec<Display>>;

    async fn get_activate_window(&self) -> Result<Option<WindowInfo>>;
}

#[derive(Clone)]
pub struct Screenshot {
    pub data: image::RgbaImage,
    pub metadata: ScreenshotMetadata,
}

impl Screenshot {
    pub fn new(data: image::RgbaImage, mode: CaptureMode, format: OutputFormat) -> Self {
        let (width, height) = data.dimensions();

        Self {
            data,
            metadata: ScreenshotMetadata {
                timestamp: chrono::Local::now(),
                mode,
                width,
                height,
                format,
                file_size: None,
            },
        }
    }

    pub fn width(&self) -> u32 {
        self.data.width()
    }

    pub fn height(&self) -> u32 {
        self.data.height()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CaptureOptions {
    pub delay: Option<std::time::Duration>,
    pub include_cursor: bool,
    pub monitor_index: Option<usize>,
    pub region: Option<Region>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale: f64,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub app_id: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn normalize(&self) -> Self {
        Self {
            x: self.x.min(self.x + self.width as i32),
            y: self.y.min(self.y + self.height as i32),
            width: self.width,
            height: self.height,
        }
    }
}

#[cfg(feature = "gui")]
pub use ui::*;
