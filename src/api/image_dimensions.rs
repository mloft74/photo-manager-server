use image::io::Reader as ImageReader;
use serde::Serialize;

use crate::api::IMAGES_DIR;

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
