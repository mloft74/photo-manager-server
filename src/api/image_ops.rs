use std::fs::File;

use image::{
    imageops::{resize, FilterType},
    ImageReader,
};
use serde::Serialize;

use crate::{
    api::{IMAGES_DIR, MAX_SCALED_IMAGE_HEIGHT, MAX_SCALED_IMAGE_WIDTH, SCALED_IMAGE_PREFIX},
    domain::models::Image,
};

const F_MAX_SCALED_IMAGE_WIDTH: f32 = MAX_SCALED_IMAGE_WIDTH as f32;
const F_MAX_SCALED_IMAGE_HEIGHT: f32 = MAX_SCALED_IMAGE_HEIGHT as f32;
const ASPECT_RATIO: f32 = F_MAX_SCALED_IMAGE_WIDTH / F_MAX_SCALED_IMAGE_HEIGHT;

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

    let resize_dimensions = compute_resize_dimensions(Dimensions {
        width: image.width,
        height: image.height,
    });
    let resized = resize(
        &decoded,
        resize_dimensions.width,
        resize_dimensions.height,
        FilterType::Lanczos3,
    );
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

pub fn scale_images(images: &[Image]) -> Result<(), Vec<ScaleImageError>> {
    let errs: Vec<_> = images
        .iter()
        .map(scale_image)
        .filter(Result::is_err)
        .map(Result::unwrap_err)
        .collect();
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

#[derive(PartialEq, Eq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

pub fn compute_resize_dimensions(dim: Dimensions) -> Dimensions {
    let dim = FDimensions::from(dim);
    let aspect_ratio = dim.width / dim.height;
    if aspect_ratio > ASPECT_RATIO {
        Dimensions {
            width: MAX_SCALED_IMAGE_WIDTH,
            height: compute_resized_height(dim),
        }
    } else {
        Dimensions {
            width: compute_resized_width(dim),
            height: MAX_SCALED_IMAGE_HEIGHT,
        }
    }
}

struct FDimensions {
    width: f32,
    height: f32,
}

impl From<Dimensions> for FDimensions {
    fn from(value: Dimensions) -> Self {
        Self {
            width: value.width as f32,
            height: value.height as f32,
        }
    }
}

fn compute_resized_width(dim: FDimensions) -> u32 {
    let scale = F_MAX_SCALED_IMAGE_HEIGHT / dim.height;
    let scaled_width = dim.width * scale;
    scaled_width.round() as u32
}

fn compute_resized_height(dim: FDimensions) -> u32 {
    let scale = F_MAX_SCALED_IMAGE_WIDTH / dim.width;
    let scaled_height = dim.height * scale;
    scaled_height.round() as u32
}
