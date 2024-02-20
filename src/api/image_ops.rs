use std::fs::File;

use image::{
    imageops::{resize, FilterType},
    io::Reader as ImageReader,
};
use serde::Serialize;

use crate::{
    api::{IMAGES_DIR, SCALED_IMAGE_PREFIX},
    domain::models::Image,
};

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScaleImageErrorType {
    ErrorOpeningImage,
    UnknownFormat,
    ImageError,
    CreateFileError,
    SaveImageError,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaleImageError {
    details: String,
    error_type: ScaleImageErrorType,
}

pub fn scale_image(image: &Image) -> Result<(), ScaleImageError> {
    let path = std::path::Path::new(IMAGES_DIR).join(&image.file_name);
    let fs_image = ImageReader::open(path).map_err(|e| ScaleImageError {
        details: e.to_string(),
        error_type: ScaleImageErrorType::ErrorOpeningImage,
    })?;
    let format = fs_image.format().ok_or_else(|| ScaleImageError {
        details: image.file_name.clone(),
        error_type: ScaleImageErrorType::UnknownFormat,
    })?;

    let decoded = fs_image.decode().map_err(|e| ScaleImageError {
        details: e.to_string(),
        error_type: ScaleImageErrorType::ImageError,
    })?;

    let resized = resize(&decoded, 1920, 1080, FilterType::Lanczos3);
    let mut file = File::create(format!(
        "{}/{}{}",
        IMAGES_DIR, SCALED_IMAGE_PREFIX, &image.file_name
    ))
    .map_err(|e| ScaleImageError {
        details: e.to_string(),
        error_type: ScaleImageErrorType::CreateFileError,
    })?;
    resized
        .write_to(&mut file, format)
        .map_err(|e| ScaleImageError {
            details: e.to_string(),
            error_type: ScaleImageErrorType::SaveImageError,
        })?;

    Ok(())
}

pub fn scale_images(images: &[&Image]) -> Result<(), Vec<ScaleImageError>> {
    let errors: Vec<_> = images
        .iter()
        .map(|i| scale_image(i))
        .filter(Result::is_err)
        .map(Result::unwrap_err)
        .collect();
    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
    }
}
