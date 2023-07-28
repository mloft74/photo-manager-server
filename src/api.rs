use axum::{middleware, Router};

use crate::{
    domain::screen_saver_manager::ScreenSaverManager,
    persistence::persistence_manager::PersistenceManager,
};

mod canon;
mod image_dimensions;
mod image_server;
mod request_tracing;
mod routing;

const IMAGES_DIR: &str = "/var/lib/photo_manager_server/images";

pub async fn make_api_router(persistence_manager: &PersistenceManager) -> Router {
    let mut manager = ScreenSaverManager::new();
    canon::update_canon(
        &persistence_manager.make_image_canon_updater(),
        &mut manager,
    )
    .await
    .expect("Canon should be updatable from startup");

    let image_server_router = image_server::create_image_server_router();

    let demo_router = routing::make_api_router(persistence_manager, manager.clone());

    Router::new()
        .merge(image_server_router)
        .merge(demo_router)
        .layer(middleware::from_fn(request_tracing::print_request_response))
}
