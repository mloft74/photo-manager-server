use std::fs;

use serde::Serialize;

use crate::{api::IMAGES_DIR, domain::models::Image};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveImageError {
    pub image_name: String,
    pub details: String,
}

pub fn remove_images(images: &[Image]) -> Result<(), Vec<RemoveImageError>> {
    let results = images.iter().map(|i| {
        let path = format!("{}/{}", IMAGES_DIR, &i.file_name);
        fs::remove_file(path).map_err(|e| RemoveImageError {
            image_name: i.file_name.clone(),
            details: e.to_string(),
        })
    });
    let errs: Vec<_> = results
        .filter(Result::is_err)
        .map(Result::unwrap_err)
        .collect();
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}
