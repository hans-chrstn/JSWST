use thiserror::Error;

pub type Result<T> = std::result::Result<T, ScreenshotError>;

#[derive(Error, Debug)]
pub enum ScreenshotError {
    #[error("Capture failed: {0}")]
    CaptureFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Portal error: {0}")]
    Portal(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Configuration parse error: {0}")]
    ConfigParse(String),

    #[error("Invalid region: {0}")]
    InvalidRegion(String),

    #[error("No display found")]
    NoDisplay,

    #[error("Backend not available")]
    BackendUnavailable,

    #[error("Operation cancelled")]
    Cancelled,

    #[cfg(feature = "gui")]
    #[error("GUI error: {0}")]
    Gui(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<anyhow::Error> for ScreenshotError {
    fn from(err: anyhow::Error) -> Self {
        ScreenshotError::Unknown(err.to_string())
    }
}

impl From<toml::de::Error> for ScreenshotError {
    fn from(value: toml::de::Error) -> Self {
        ScreenshotError::ConfigParse(value.to_string())
    }
}

impl From<toml::ser::Error> for ScreenshotError {
    fn from(value: toml::ser::Error) -> Self {
        ScreenshotError::ConfigParse(value.to_string())
    }
}

impl From<serde_json::Error> for ScreenshotError {
    fn from(value: serde_json::Error) -> Self {
        ScreenshotError::ConfigParse(value.to_string())
    }
}
