use std::fs;

use axum::{middleware, Router};
use futures::TryStreamExt;
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
    let manager = update_canon(action_provider.get_image_canon_updater()).await;

    let image_server_router = image_server::create_image_server_router();

    let demo_router = routing::make_api_router(action_provider);

    Router::new()
        .merge(image_server_router)
        .merge(demo_router)
        .layer(middleware::from_fn(request_tracing::print_request_response))
        .layer(TraceLayer::new_for_http())
}

async fn update_canon(canon_updater: impl ImageCanonUpdater) -> ScreenSaverManager {
    fs::create_dir_all(IMAGES_DIR).expect("Could not create images directory");
    let images_dir = fs::read_dir(IMAGES_DIR).expect("Could not read images directory");
    let mut images = Vec::new();
    for entry in images_dir {
        let entry = entry.expect("Could not get entry in images directory");
        let file_name = entry
            .file_name()
            .to_str()
            .expect("Could not convert file name to &str")
            .to_string();
        let (width, height) =
            get_image_dimensions(&file_name).expect("Could not get dimensions of image");
        images.push(Image {
            file_name,
            width,
            height,
        })
    }
    canon_updater
        .update_canon(images.iter())
        .await
        .expect("Could not update canon");

    let mut manager = ScreenSaverManager::new();
    manager.replace(images.into_iter());

    manager
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
