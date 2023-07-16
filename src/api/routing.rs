use axum::Router;

use crate::domain::{actions::ActionProvider, screen_saver_manager::ScreenSaverManager};

mod image;
mod ping;

pub fn make_api_router(
    action_provider: &(impl ActionProvider + 'static),
    manager: &ScreenSaverManager,
) -> Router {
    Router::new().nest(
        "/api",
        Router::new()
            .merge(image::make_image_router(action_provider, manager))
            .merge(ping::make_ping_router()),
    )
}
