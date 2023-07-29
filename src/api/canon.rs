use tokio::{fs, io};

use serde::Serialize;

use crate::{
    api::{
        image_dimensions::{self, FetchImageDimensionsError},
        IMAGES_DIR,
    },
    domain::{models::Image, screen_saver_manager::ScreenSaverManager},
    persistence::image::image_canon_updater::ImageCanonUpdater,
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

pub async fn fetch_canon() -> Result<Vec<Image>, FetchCanonError> {
    fs::create_dir_all(IMAGES_DIR).await?;
    let mut images_dir = fs::read_dir(IMAGES_DIR).await?;
    let mut images = Vec::new();
    while let Some(entry) = images_dir.next_entry().await? {
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
    canon_updater: &ImageCanonUpdater,
    manager: &mut ScreenSaverManager,
) -> Result<(), UpdateCanonError> {
    let images = fetch_canon().await?;
    canon_updater
        .update_canon(images.iter())
        .await
        .map_err(UpdateCanonError::FailedToUpdateCanon)?;

    manager.replace(images.into_iter());

    Ok(())
}
