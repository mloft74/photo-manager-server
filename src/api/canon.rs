use std::{fs, io};

use serde::Serialize;

use crate::{
    api::{
        image_dimensions::{self, FetchImageDimensionsError},
        IMAGES_DIR,
    },
    domain::{
        actions::images::ImageCanonUpdater, models::Image, screen_saver_manager::ScreenSaverManager,
    },
};

#[derive(Debug, Serialize)]
pub enum FetchCanonError {
    IO(String),
    FileNameConversionError,
    FetchDimensionsError(FetchImageDimensionsError),
}

impl From<io::Error> for FetchCanonError {
    fn from(value: io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

impl From<FetchImageDimensionsError> for FetchCanonError {
    fn from(value: FetchImageDimensionsError) -> Self {
        Self::FetchDimensionsError(value)
    }
}

pub fn fetch_canon() -> Result<Vec<Image>, FetchCanonError> {
    fs::create_dir_all(IMAGES_DIR)?;
    let images_dir = fs::read_dir(IMAGES_DIR)?;
    let mut images = Vec::new();
    for entry in images_dir {
        let entry = entry?;
        let file_name = entry
            .file_name()
            .to_str()
            .ok_or(FetchCanonError::FileNameConversionError)?
            .to_string();
        let (width, height) = image_dimensions::fetch_image_dimensions(&file_name)?;
        images.push(Image {
            file_name,
            width,
            height,
        })
    }
    Ok(images)
}

#[derive(Debug, Serialize)]
pub enum UpdateCanonError {
    FetchCanonError(FetchCanonError),
    FailedToUpdateCanon(String),
}

impl From<FetchCanonError> for UpdateCanonError {
    fn from(value: FetchCanonError) -> Self {
        Self::FetchCanonError(value)
    }
}

pub async fn update_canon(
    canon_updater: &impl ImageCanonUpdater,
    manager: &mut ScreenSaverManager,
) -> Result<(), UpdateCanonError> {
    let images = fetch_canon()?;
    canon_updater
        .update_canon(images.iter())
        .await
        .map_err(UpdateCanonError::FailedToUpdateCanon)?;

    manager.replace(images.into_iter());

    Ok(())
}
