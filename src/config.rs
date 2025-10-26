use crate::{CaptureMode, OutputFormat, Result};
use serde::{Deserialize, Serialize};
use std::fs;
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
        let pictures_dir = directories::UserDirs::new()
            .and_then(|dirs| dirs.picture_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("~/Pictures"));

        #[cfg(feature = "gui")]
        let gui = GuiConfig {
            animation: AnimationConfig {
                duration_ms: 800,
                easing: "ease-in-out".to_string(),
                start_shape: "circle".to_string(),
                end_shape: "rounded-rect".to_string(),
            },
            css_classes: {
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
            },
            editor_enabled: true,
        };

        Self {
            default_mode: CaptureMode::Region,
            default_format: OutputFormat::Png,
            save_directory: pictures_dir,
            filename_template: "screenshot_%Y%m%d_%H%M%S".to_string(),
            auto_copy_to_clipboard: false,
            delay_seconds: 0,
            include_cursor: false,

            #[cfg(feature = "gui")]
            gui,

            shortcuts: ShortcutConfig {
                save: "space".to_string(),
                cancel: "Escape".to_string(),
                undo: "Ctrl+z".to_string(),
                redo: "Ctrl+y".to_string(),
                copy: "Ctrl+c".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = directories::ProjectDirs::from(
            "com",
            "wayland",
            "just-a-simple-wayland-screenshot-tool",
        )
        .ok_or_else(|| {
            crate::error::ScreenshotError::Config("Cannot determine config directory".to_string())
        })?;

        let config_path = config_dir.config_dir().join("config.toml");

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = directories::ProjectDirs::from(
            "com",
            "wayland",
            "just-a-simple-wayland-screenshot-tool",
        )
        .ok_or_else(|| {
            crate::error::ScreenshotError::Config("Cannot determine config directory".to_string())
        })?;

        fs::create_dir_all(config_dir.config_dir())?;
        let config_path = config_dir.config_dir().join("config.toml");

        let contents = toml::to_string_pretty(self)?;

        fs::write(config_path, contents)?;
        Ok(())
    }

    pub fn generate_filename(&self) -> String {
        let now = chrono::Local::now();
        now.format(&self.filename_template).to_string()
    }
}
