use std::{collections::HashMap, ffi::OsString, fs, io};

use serde::Serialize;

use crate::{
    api::{
        image_ops::{
            self, compute_resize_dimensions, scale_images, Dimensions, FetchImageDimensionsError,
            ScaleImageError,
        },
        IMAGES_DIR, SCALED_IMAGE_PREFIX,
    },
    domain::{actions::image::UpdateCanon, models::Image, screensaver::Screensaver},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateCanonError {
    FetchCanonError(FetchCanonError),
    FailedToUpdateCanon(String),
    FailedToRemoveInvalidImages(Vec<String>),
    FailedToScaleImages(Vec<ScaleImageError>),
}

impl From<FetchCanonError> for UpdateCanonError {
    fn from(value: FetchCanonError) -> Self {
        Self::FetchCanonError(value)
    }
}

impl From<Vec<io::Error>> for UpdateCanonError {
    fn from(value: Vec<io::Error>) -> Self {
        Self::FailedToRemoveInvalidImages(value.into_iter().map(|e| e.to_string()).collect())
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

    let partition = separate_canon(images);
    let canon = partition.canon.clone();
    let data = pair_canon_with_scaled(partition);
    remove_images(data.scale_without_canon)?;
    let invalid = find_invalid_scaled(data.pairs);
    let (canons_needing_new_scales, invalid_scales): (Vec<_>, Vec<_>) =
        invalid.into_iter().map(|p| (p.canon, p.scale)).unzip();
    remove_images(invalid_scales)?;
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

struct CanonPartition {
    canon: Vec<Image>,
    scaled: Vec<Image>,
}
fn separate_canon(images: Vec<Image>) -> CanonPartition {
    let (scaled, canon) = images
        .into_iter()
        .partition(|i| i.file_name.starts_with(SCALED_IMAGE_PREFIX));
    CanonPartition { canon, scaled }
}

struct CanonScale {
    canon: Image,
    scale: Image,
}
struct CanonScaledPairData {
    pairs: Vec<CanonScale>,
    canon_without_scale: Vec<Image>,
    scale_without_canon: Vec<Image>,
}
fn pair_canon_with_scaled(partition: CanonPartition) -> CanonScaledPairData {
    let mut pairs = Vec::new();
    let mut canon_without_scale = Vec::new();
    let mut scaled: HashMap<String, Image> = partition
        .scaled
        .into_iter()
        .map(|i| (i.file_name.clone(), i))
        .collect();
    for canon in partition.canon {
        let scaled_name = format!("{}{}", SCALED_IMAGE_PREFIX, &canon.file_name);
        let existing = scaled.remove(&scaled_name);
        if let Some(scale) = existing {
            pairs.push(CanonScale { canon, scale });
        } else {
            canon_without_scale.push(canon);
        }
    }

    let scale_without_canon: Vec<_> = scaled.into_values().collect();

    CanonScaledPairData {
        pairs,
        canon_without_scale,
        scale_without_canon,
    }
}

fn find_invalid_scaled(images: Vec<CanonScale>) -> Vec<CanonScale> {
    images
        .into_iter()
        .filter(|i| {
            let expected = compute_resize_dimensions(Dimensions {
                width: i.canon.width,
                height: i.canon.height,
            });
            let actual = Dimensions {
                width: i.scale.width,
                height: i.scale.height,
            };
            expected != actual
        })
        .collect()
}

fn remove_images(images: Vec<Image>) -> Result<(), Vec<io::Error>> {
    let results = images.into_iter().map(|i| fs::remove_file(i.file_name));
    let errs: Vec<_> = results.filter(Result::is_err).collect();
    if errs.is_empty() {
        Err(errs.into_iter().map(Result::unwrap_err).collect())
    } else {
        Ok(())
    }
}
