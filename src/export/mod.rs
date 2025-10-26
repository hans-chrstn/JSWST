use crate::{OutputFormat, Result, Screenshot, ScreenshotError};
use std::path::Path;

pub struct Exporter;

impl Exporter {
    pub fn save<P: AsRef<Path>>(
        screenshot: &Screenshot,
        path: P,
        format: OutputFormat,
    ) -> Result<u64> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        match format {
            OutputFormat::Png => {
                screenshot
                    .data
                    .save_with_format(path, image::ImageFormat::Png)?;
            }
            OutputFormat::Jpeg => {
                let rgb = image::DynamicImage::ImageRgba8(screenshot.data.clone()).to_rgb8();
                rgb.save_with_format(path, image::ImageFormat::Jpeg)?;
            }
            OutputFormat::Webp => {
                screenshot
                    .data
                    .save_with_format(path, image::ImageFormat::WebP)?;
            }
            OutputFormat::Clipboard => {
                return Err(ScreenshotError::Config(
                    "Use copy_to_clipboard instead".to_string(),
                ));
            }
        }

        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len())
    }

    pub fn copy_to_clipboard(screenshot: &Screenshot) -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("wst_clipboard.png");

        screenshot
            .data
            .save_with_format(&temp_file, image::ImageFormat::Png)?;

        let output = std::process::Command::new("wl-copy")
            .arg("--type")
            .arg("image/png")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let bytes = std::fs::read(&temp_file)?;
                    stdin.write_all(&bytes)?;
                }
                child.wait()
            });

        match output {
            Ok(status) if status.success() => Ok(()),
            _ => Err(ScreenshotError::Config("wl-copy not available".to_string())),
        }
    }

    pub fn export_metadata<P: AsRef<Path>>(screenshot: &Screenshot, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(&screenshot.metadata)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
