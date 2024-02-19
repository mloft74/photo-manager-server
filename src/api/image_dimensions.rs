use image::{
    imageops::{resize, FilterType},
    io::Reader as ImageReader,
};
use serde::Serialize;

use crate::{api::IMAGES_DIR, domain::models::Image};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FetchImageDimensionsError {
    ErrorOpeningImage(String),
    FailedToGetDimensions(String),
}

pub fn fetch_image_dimensions(file_name: &str) -> Result<(u32, u32), FetchImageDimensionsError> {
    let path = std::path::Path::new(IMAGES_DIR).join(file_name);
    let image = ImageReader::open(path)
        .map_err(|e| FetchImageDimensionsError::ErrorOpeningImage(e.to_string()))?;
    let dim = image
        .into_dimensions()
        .map_err(|e| FetchImageDimensionsError::FailedToGetDimensions(e.to_string()))?;

    Ok(dim)
}

pub enum ScaleImageError {
    ErrorOpeningImage(String),
    UnknownFormat(String),
    ImageError(String),
}

pub fn scale_image(image: &Image) -> Result<(), ScaleImageError> {
    let path = std::path::Path::new(IMAGES_DIR).join(&image.file_name);
    let fs_image =
        ImageReader::open(path).map_err(|e| ScaleImageError::ErrorOpeningImage(e.to_string()))?;
    let format = fs_image
        .format()
        .ok_or_else(|| ScaleImageError::UnknownFormat(image.file_name.clone()))?;

    let decoded = fs_image
        .decode()
        .map_err(|e| ScaleImageError::ImageError(e.to_string()))?;

    let resized = resize(&decoded, 1920, 1080, FilterType::Lanczos3);

    Ok(())
}

pub fn scale_images(images: &[Image]) {}
