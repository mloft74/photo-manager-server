use std::{fs, io};

use axum::{middleware, Router};
use image::io::Reader as ImageReader;
use serde::Serialize;
use tower_http::trace::TraceLayer;

use crate::domain::{
    actions::{images::ImageCanonUpdater, ActionProvider},
    models::Image,
    screen_saver_manager::ScreenSaverManager,
};

mod image_server;
mod request_tracing;
mod routing;

const IMAGES_DIR: &str = "/var/lib/photo_manager_server/images";

pub async fn make_api_router(action_provider: &(impl ActionProvider + 'static)) -> Router {
    let mut manager = ScreenSaverManager::new();
    update_canon(&action_provider.get_image_canon_updater(), &mut manager)
        .await
        .expect("Could not update canon");

    let image_server_router = image_server::create_image_server_router();

    let demo_router = routing::make_api_router(action_provider);

    Router::new()
        .merge(image_server_router)
        .merge(demo_router)
        .layer(middleware::from_fn(request_tracing::print_request_response))
        .layer(TraceLayer::new_for_http())
}

#[derive(Debug, Serialize)]
enum FetchCanonError {
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

fn fetch_canon() -> Result<Vec<Image>, FetchCanonError> {
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
        let (width, height) = fetch_image_dimensions(&file_name)?;
        images.push(Image {
            file_name,
            width,
            height,
        })
    }
    Ok(images)
}

#[derive(Debug, Serialize)]
enum UpdateCanonError {
    FetchCanonError(FetchCanonError),
    FailedToUpdateCanon(String),
}

impl From<FetchCanonError> for UpdateCanonError {
    fn from(value: FetchCanonError) -> Self {
        Self::FetchCanonError(value)
    }
}

async fn update_canon(
    canon_updater: &impl ImageCanonUpdater,
    manager: &mut ScreenSaverManager,
) -> Result<(), UpdateCanonError> {
    let images = fetch_canon()?;
    canon_updater
        .update_canon(images.iter())
        .await
        .map_err(|e| UpdateCanonError::FailedToUpdateCanon(e.to_string()))?;

    manager.replace(images.into_iter());

    Ok(())
}

#[derive(Debug, Serialize)]
enum FetchImageDimensionsError {
    ErrorOpeningImage(String),
    FailedToGetDimensions(String),
}

fn fetch_image_dimensions(file_name: &str) -> Result<(u32, u32), FetchImageDimensionsError> {
    let path = std::path::Path::new(IMAGES_DIR).join(file_name);
    let image = ImageReader::open(path)
        .map_err(|e| FetchImageDimensionsError::ErrorOpeningImage(e.to_string()))?;
    let dim = image
        .into_dimensions()
        .map_err(|e| FetchImageDimensionsError::FailedToGetDimensions(e.to_string()))?;

    Ok(dim)
}
