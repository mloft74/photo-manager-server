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
    update_canon(&action_provider.get_image_canon_updater(), &mut manager).await;

    let image_server_router = image_server::create_image_server_router();

    let demo_router = routing::make_api_router(action_provider);

    Router::new()
        .merge(image_server_router)
        .merge(demo_router)
        .layer(middleware::from_fn(request_tracing::print_request_response))
        .layer(TraceLayer::new_for_http())
}

#[derive(Debug, Serialize)]
enum GetCanonError {
    IO(String),
    FileNameConversionError,
    ImageDimensions(GetImageDimensionsError),
}

impl From<io::Error> for GetCanonError {
    fn from(value: io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

impl From<GetImageDimensionsError> for GetCanonError {
    fn from(value: GetImageDimensionsError) -> Self {
        Self::ImageDimensions(value)
    }
}

fn get_canon() -> Result<Vec<Image>, GetCanonError> {
    fs::create_dir_all(IMAGES_DIR)?;
    let images_dir = fs::read_dir(IMAGES_DIR)?;
    let mut images = Vec::new();
    for entry in images_dir {
        let entry = entry?;
        let file_name = entry
            .file_name()
            .to_str()
            .ok_or(GetCanonError::FileNameConversionError)?
            .to_string();
        let (width, height) = get_image_dimensions(&file_name)?;
        images.push(Image {
            file_name,
            width,
            height,
        })
    }
    Ok(images)
}

async fn update_canon(canon_updater: &impl ImageCanonUpdater, manager: &mut ScreenSaverManager) {
    let images = get_canon().expect("Could not get canon");
    canon_updater
        .update_canon(images.iter())
        .await
        .expect("Could not update canon");

    manager.replace(images.into_iter());
}

#[derive(Debug, Serialize)]
enum GetImageDimensionsError {
    ErrorOpeningImage(String),
    FailedToGetDimensions(String),
}

fn get_image_dimensions(file_name: &str) -> Result<(u32, u32), GetImageDimensionsError> {
    let path = std::path::Path::new(IMAGES_DIR).join(file_name);
    let image = ImageReader::open(path)
        .map_err(|e| GetImageDimensionsError::ErrorOpeningImage(e.to_string()))?;
    let dim = image
        .into_dimensions()
        .map_err(|e| GetImageDimensionsError::FailedToGetDimensions(e.to_string()))?;

    Ok(dim)
}
