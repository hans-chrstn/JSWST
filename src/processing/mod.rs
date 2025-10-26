use crate::{Result, Screenshot};
use image::{Rgba, RgbaImage};

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn crop(
        screenshot: &Screenshot,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<Screenshot> {
        let cropped = image::imageops::crop_imm(&screenshot.data, x, y, width, height).to_image();

        let mut new_screenshot = screenshot.clone();
        new_screenshot.data = cropped;
        new_screenshot.metadata.width = width;
        new_screenshot.metadata.height = height;

        Ok(new_screenshot)
    }

    pub fn add_border(screenshot: &Screenshot, width: u32, color: Rgba<u8>) -> Result<Screenshot> {
        let old_width = screenshot.width();
        let old_height = screenshot.height();
        let new_width = old_width + 2 * width;
        let new_height = old_height + 2 * width;

        let mut new_image = RgbaImage::from_pixel(new_width, new_height, color);

        image::imageops::overlay(&mut new_image, &screenshot.data, width as i64, width as i64);

        let mut new_screenshot = screenshot.clone();
        new_screenshot.data = new_image;
        new_screenshot.metadata.width = new_width;
        new_screenshot.metadata.height = new_height;

        Ok(new_screenshot)
    }

    pub fn add_shadow(screenshot: &Screenshot, offset: u32) -> Result<Screenshot> {
        let width = screenshot.width();
        let height = screenshot.height();
        let new_width = width + offset * 2;
        let new_height = height + offset * 2;

        let mut new_image = RgbaImage::from_pixel(new_width, new_height, Rgba([0, 0, 0, 0]));

        image::imageops::overlay(
            &mut new_image,
            &screenshot.data,
            offset as i64,
            offset as i64,
        );

        let mut new_screenshot = screenshot.clone();
        new_screenshot.data = new_image;
        new_screenshot.metadata.width = new_width;
        new_screenshot.metadata.height = new_height;

        Ok(new_screenshot)
    }

    pub fn resize(screenshot: &Screenshot, width: u32, height: u32) -> Result<Screenshot> {
        let resized = image::imageops::resize(
            &screenshot.data,
            width,
            height,
            image::imageops::FilterType::Lanczos3,
        );

        let mut new_screenshot = screenshot.clone();
        new_screenshot.data = resized;
        new_screenshot.metadata.width = width;
        new_screenshot.metadata.height = height;

        Ok(new_screenshot)
    }

    pub fn blur(screenshot: &Screenshot, sigma: f32) -> Result<Screenshot> {
        let blurred = image::imageops::blur(&screenshot.data, sigma);

        let mut new_screenshot = screenshot.clone();
        new_screenshot.data = blurred;

        Ok(new_screenshot)
    }
}
