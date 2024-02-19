use std::{ffi::OsString, fs, io};

use serde::Serialize;

use crate::{
    api::{
        image_dimensions::{self, FetchImageDimensionsError},
        IMAGES_DIR,
    },
    domain::{actions::image::UpdateCanon, models::Image, screensaver::Screensaver},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FetchCanonError {
    IO(String),
    MultiIO(Vec<String>),
    FileNameConversionsError(Vec<OsString>),
    FetchDimensionsErrors(Vec<FetchDimensionsError>),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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

    let images_dir = fs::read_dir(IMAGES_DIR)?;
    let (oks, errs): (Vec<_>, Vec<_>) = images_dir.into_iter().partition(Result::is_ok);
    if !errs.is_empty() {
        let errs: Vec<_> = errs
            .into_iter()
            .map(Result::unwrap_err)
            .map(|e| e.to_string())
            .collect();
        return Err(FetchCanonError::MultiIO(errs));
    }

    let file_name_results = oks.into_iter().map(|res| {
        let entry = res.unwrap();
        let file_name = entry.file_name();
        let name_opt = file_name.to_str().map(|n| n.to_string());
        name_opt.ok_or(file_name)
    });

    let (oks, errs): (Vec<_>, Vec<_>) = file_name_results.partition(Result::is_ok);
    if !errs.is_empty() {
        let errs: Vec<_> = errs.into_iter().map(Result::unwrap_err).collect();
        return Err(FetchCanonError::FileNameConversionsError(errs));
    }

    let image_results = oks
        .into_iter()
        .map(Result::unwrap)
        .map(|n| fetch_dimensions(&n));
    let (oks, errs): (Vec<_>, Vec<_>) = image_results.partition(Result::is_ok);
    if errs.is_empty() {
        Ok(oks.into_iter().map(Result::unwrap).collect())
    } else {
        Err(FetchCanonError::FetchDimensionsErrors(
            errs.into_iter().map(Result::unwrap_err).collect(),
        ))
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
#[serde(rename_all = "camelCase")]
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
    uc: &impl UpdateCanon,
    screensaver: &mut impl Screensaver,
) -> Result<(), UpdateCanonError> {
    let images = fetch_canon()?;
    uc.update_canon(images.iter())
        .await
        .map_err(UpdateCanonError::FailedToUpdateCanon)?;

    screensaver.replace(
        images
            .into_iter()
            .map(|i| (i.file_name.clone(), i))
            .collect(),
    );

    Ok(())
}
