use std::{fs, io};

use serde::Serialize;

use crate::{
    api::{
        image_dimensions::{self, FetchImageDimensionsError},
        IMAGES_DIR,
    },
    domain::{
        actions::image::UpdateCanon, models::Image, screen_saver_manager::ScreenSaverManager,
    },
};

#[derive(Debug, Serialize)]
pub enum FetchCanonError {
    IO(String),
    MultiIO(Vec<String>),
    FileNameConversionError,
    FetchDimensionsErrors(Vec<FetchDimensionsError>),
}

#[derive(Debug, Serialize)]
pub struct FetchDimensionsError {
    file_name: String,
    err: FetchImageDimensionsError,
}

impl From<io::Error> for FetchCanonError {
    fn from(value: io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

impl From<(String, FetchImageDimensionsError)> for FetchDimensionsError {
    fn from((file_name, err): (String, FetchImageDimensionsError)) -> Self {
        Self { file_name, err }
    }
}

impl From<(FetchImageDimensionsError, String)> for FetchDimensionsError {
    fn from((err, file_name): (FetchImageDimensionsError, String)) -> Self {
        Self { file_name, err }
    }
}

fn fetch_canon() -> Result<Vec<Image>, FetchCanonError> {
    fs::create_dir_all(IMAGES_DIR)?;

    let images_dir: Vec<_> = fs::read_dir(IMAGES_DIR)?.collect();
    let (oks, errs): (Vec<_>, Vec<_>) = images_dir.into_iter().partition(Result::is_ok);
    if !errs.is_empty() {
        let errs: Vec<_> = errs
            .into_iter()
            .map(Result::unwrap_err)
            .map(|e| e.to_string())
            .collect();
        return Err(FetchCanonError::MultiIO(errs));
    }

    let images_dir: Vec<_> = oks.into_iter().map(Result::unwrap).collect();

    let mut oks = Vec::new();
    let mut errs = Vec::new();
    for entry in images_dir {
        let file_name = entry
            .file_name()
            .to_str()
            .ok_or(FetchCanonError::FileNameConversionError)?
            .to_string();
        match fetch_dimensions(&file_name) {
            Ok(v) => oks.push(v),
            Err(e) => errs.push(e),
        }
    }
    if errs.is_empty() {
        Ok(oks)
    } else {
        Err(FetchCanonError::FetchDimensionsErrors(errs))
    }
}

fn fetch_dimensions(file_name: &str) -> Result<Image, FetchDimensionsError> {
    let (width, height) = image_dimensions::fetch_image_dimensions(file_name)
        .map_err(|e| (file_name.to_string(), e))?;
    Ok(Image {
        file_name: file_name.to_string(),
        width,
        height,
    })
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
    update_canon_op: &impl UpdateCanon,
    mngr: &mut ScreenSaverManager,
) -> Result<(), UpdateCanonError> {
    let images = fetch_canon()?;
    update_canon_op
        .update_canon(images.iter())
        .await
        .map_err(UpdateCanonError::FailedToUpdateCanon)?;

    mngr.replace(images.into_iter());

    Ok(())
}
