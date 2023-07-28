use axum::Router;

use crate::{
    domain::screen_saver_manager::ScreenSaverManager,
    persistence::persistence_manager::PersistenceManager,
};

mod image;
mod ping;

pub fn make_api_router(
    persistence_manager: &PersistenceManager,
    manager: ScreenSaverManager,
) -> Router {
    Router::new().nest(
        "/api",
        Router::new()
            .merge(image::make_image_router(
                persistence_manager,
                manager.clone(),
            ))
            .merge(ping::make_ping_router()),
    )
}
