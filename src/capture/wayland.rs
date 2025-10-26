use crate::{
    CaptureMode, CaptureOptions, Display, OutputFormat, Result, Screenshot, ScreenshotBackend,
    ScreenshotError, WindowInfo,
};
use async_trait::async_trait;
use image::RgbaImage;

pub struct WaylandBackend;

impl WaylandBackend {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    async fn capture_via_portal(&self, interactive: bool) -> Result<RgbaImage> {
        use ashpd::desktop::screenshot::ScreenshotRequest;

        let response = ScreenshotRequest::default()
            .interactive(interactive)
            .send()
            .await
            .map_err(|e| ScreenshotError::Portal(e.to_string()))?
            .response()
            .map_err(|e| ScreenshotError::Portal(e.to_string()))?;

        let uri = response.uri();
        let path = uri
            .to_file_path()
            .map_err(|_| ScreenshotError::Portal("Invalid file path".to_string()))?;

        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| ScreenshotError::Io(e))?;

        let img = image::load_from_memory(&bytes).map_err(ScreenshotError::Image)?;

        let _ = tokio::fs::remove_file(path).await;

        Ok(img.to_rgba8())
    }
}

#[async_trait]
impl ScreenshotBackend for WaylandBackend {
    async fn capture(&self, mode: CaptureMode, options: &CaptureOptions) -> Result<Screenshot> {
        if let Some(delay) = options.delay {
            tokio::time::sleep(delay).await;
        }

        let data = match mode {
            CaptureMode::Screen => self.capture_via_portal(false).await?,
            CaptureMode::Window | CaptureMode::Region => self.capture_via_portal(true).await?,
            CaptureMode::Monitor => self.capture_via_portal(false).await?,
        };

        let data = if let Some(region) = options.region {
            let region = region.normalize();

            if region.x < 0
                || region.y < 0
                || region.x as u32 + region.width > data.width()
                || region.y as u32 + region.height > data.height()
            {
                return Err(ScreenshotError::InvalidRegion(
                    "Region out of bounds".to_string(),
                ));
            }

            image::imageops::crop_imm(
                &data,
                region.x as u32,
                region.y as u32,
                region.width,
                region.height,
            )
            .to_image()
        } else {
            data
        };

        Ok(Screenshot::new(data, mode, OutputFormat::Png))
    }

    async fn get_displays(&self) -> Result<Vec<Display>> {
        Ok(vec![Display {
            name: "Primary Display".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            scale: 1.0,
            is_primary: true,
        }])
    }

    async fn get_activate_window(&self) -> Result<Option<WindowInfo>> {
        Ok(None)
    }
}
