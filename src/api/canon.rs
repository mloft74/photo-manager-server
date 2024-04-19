use std::{ffi::OsString, fs, io};

use serde::Serialize;

use crate::{
    api::{
        canon::remove_images::RemoveImageError,
        image_ops::{self, scale_images, FetchImageDimensionsError, ScaleImageError},
        IMAGES_DIR,
    },
    domain::{actions::image::UpdateCanon, models::Image, screensaver::Screensaver},
};

mod logging;
mod remove_images;
mod scaling;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateCanonError {
    FetchCanonError(FetchCanonError),
    FailedToUpdateCanon(String),
    FailedToRemoveInvalidImages(Vec<RemoveImageError>),
    FailedToScaleImages(Vec<ScaleImageError>),
}

impl From<FetchCanonError> for UpdateCanonError {
    fn from(value: FetchCanonError) -> Self {
        Self::FetchCanonError(value)
    }
}

impl From<Vec<RemoveImageError>> for UpdateCanonError {
    fn from(value: Vec<RemoveImageError>) -> Self {
        Self::FailedToRemoveInvalidImages(value)
    }
}

impl From<Vec<ScaleImageError>> for UpdateCanonError {
    fn from(value: Vec<ScaleImageError>) -> Self {
        Self::FailedToScaleImages(value)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FetchCanonError {
    IO(String),
    MultiIO(Vec<String>),
    FileNameConversionsError(Vec<OsString>),
    FetchDimensionsErrors(Vec<FetchDimensionsError>),
}

impl From<io::Error> for FetchCanonError {
    fn from(value: io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchDimensionsError {
    file_name: String,
    err: FetchImageDimensionsError,
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

pub async fn update_canon(
    uc: &impl UpdateCanon,
    screensaver: &mut impl Screensaver,
) -> Result<(), UpdateCanonError> {
    let images = fetch_images()?;
    logging::log_images(&images);

    let partition = scaling::separate_canon(images);
    let canon = partition.canon.clone();
    let data = scaling::pair_canon_with_scaled(partition);

    logging::log_canon_scaled_pair_data(&data);

    remove_images::remove_images(data.scale_without_canon)?;

    let invalid = scaling::find_invalid_pairs(data.pairs);
    let (canons_needing_new_scales, invalid_scales): (Vec<_>, Vec<_>) =
        invalid.into_iter().map(|p| (p.canon, p.scale)).unzip();
    logging::log_needing_new_scales(&canons_needing_new_scales);
    logging::log_invalid_scales(&invalid_scales);

    remove_images::remove_images(invalid_scales)?;
    let images_needing_scaling: Vec<_> = data
        .canon_without_scale
        .into_iter()
        .chain(canons_needing_new_scales)
        .collect();
    scale_images(&images_needing_scaling)?;

    uc.update_canon(canon.iter())
        .await
        .map_err(UpdateCanonError::FailedToUpdateCanon)?;

    screensaver.replace(
        canon
            .into_iter()
            .map(|i| (i.file_name.clone(), i))
            .collect(),
    );

    Ok(())
}

fn fetch_images() -> Result<Vec<Image>, FetchCanonError> {
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

    let file_names = oks.into_iter().map(Result::unwrap);
    let image_results = file_names.into_iter().map(|n| fetch_dimensions(&n));

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
    let (width, height) =
        image_ops::fetch_image_dimensions(file_name).map_err(|e| (file_name.to_string(), e))?;
    Ok(Image {
        file_name: file_name.to_string(),
        width,
        height,
    })
}
