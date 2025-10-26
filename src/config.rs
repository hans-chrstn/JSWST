use crate::{CaptureMode, OutputFormat, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_mode: CaptureMode,
    pub default_format: OutputFormat,
    pub save_directory: PathBuf,
    pub filename_template: String,
    pub auto_copy_to_clipboard: bool,
    pub delay_seconds: u64,
    pub include_cursor: bool,

    #[cfg(feature = "gui")]
    pub gui: GuiConfig,

    pub shortcuts: ShortcutConfig,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    pub animation: AnimationConfig,
    pub css_classes: std::collections::HashMap<String, String>,
    pub editor_enabled: bool,
}

#[cfg(feature = "gui")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub duration_ms: u64,
    pub easing: String,
    pub start_shape: String,
    pub end_shape: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub save: String,
    pub cancel: String,
    pub undo: String,
    pub redo: String,
    pub copy: String,
}

impl Default for Config {
    fn default() -> Self {
        let pictures_dir = Self::get_pictures_directory();

        Self {
            default_mode: CaptureMode::Region,
            default_format: OutputFormat::Png,
            save_directory: pictures_dir,
            filename_template: "screenshot_%Y%m%d_%H%M%S".to_string(),
            auto_copy_to_clipboard: false,
            delay_seconds: 0,
            include_cursor: false,

            #[cfg(feature = "gui")]
            gui: GuiConfig::default(),

            shortcuts: ShortcutConfig::default(),
        }
    }
}

impl Config {
    fn get_pictures_directory() -> PathBuf {
        directories::UserDirs::new()
            .and_then(|dirs| dirs.picture_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| {
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join("Pictures"))
                    .unwrap_or_else(|_| PathBuf::from("/tmp"))
            })
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(config_path, contents)?;
        Ok(())
    }

    fn config_file_path() -> Result<PathBuf> {
        directories::ProjectDirs::from("com", "wayland", "just-a-simple-wayland-screenshot-tool")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .ok_or_else(|| {
                crate::error::ScreenshotError::Config(
                    "Cannot determine config directory".to_string(),
                )
            })
    }

    pub fn generate_filename(&self) -> String {
        chrono::Local::now()
            .format(&self.filename_template)
            .to_string()
    }
}

#[cfg(feature = "gui")]
impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            animation: AnimationConfig::default(),
            css_classes: Self::default_css_classes(),
            editor_enabled: true,
        }
    }
}

#[cfg(feature = "gui")]
impl GuiConfig {
    fn default_css_classes() -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        map.insert(
            "circle".to_string(),
            "border-radius: 50%; background: rgba(76, 154, 255, 0.8);".to_string(),
        );
        map.insert(
            "rounded-rect".to_string(),
            "border-radius: 20px; background: rgba(76, 154, 255, 0.8);".to_string(),
        );
        map
    }
}

#[cfg(feature = "gui")]
impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration_ms: 800,
            easing: "ease-in-out".to_string(),
            start_shape: "circle".to_string(),
            end_shape: "rounded-rect".to_string(),
        }
    }
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            save: "space".to_string(),
            cancel: "Escape".to_string(),
            undo: "Ctrl+z".to_string(),
            redo: "Ctrl+y".to_string(),
            copy: "Ctrl+c".to_string(),
        }
    }
}
