use axum::{middleware, Router};

use crate::{persistence::PersistenceManager, state::screensaver_manager::ScreensaverManager};

mod canon;
mod image_ops;
mod image_server;
mod request_tracing;
mod routing;

const IMAGES_DIR: &str = "/var/lib/photo_manager_server/images";

const SCALED_IMAGE_PREFIX: &str = "photo_manager_server_scaled___";
const MAX_SCALED_IMAGE_HEIGHT: u32 = 1080;
const MAX_SCALED_IMAGE_WIDTH: u32 = 1920;

pub async fn make_api_router(persistence_mngr: &PersistenceManager) -> Router {
    let mut screensaver_mngr = ScreensaverManager::new();
    canon::update_canon(&persistence_mngr, &mut screensaver_mngr)
        .await
        .expect("Canon should be updatable from startup");

    let image_server_router = image_server::create_image_server_router();

    let demo_router = routing::make_api_router(persistence_mngr, &screensaver_mngr);

    Router::new()
        .merge(image_server_router)
        .merge(demo_router)
        .layer(middleware::from_fn(request_tracing::print_request_response))
}
