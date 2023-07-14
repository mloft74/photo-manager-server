use axum::Router;

use crate::domain::{actions::ActionProvider, screen_saver_manager::ScreenSaverManager};

mod get;
mod update_canon;
mod upload;

pub fn make_image_router(
    action_provider: &(impl ActionProvider + 'static),
    manager: &ScreenSaverManager,
) -> Router {
    Router::new().nest(
        "/image",
        Router::new()
            .merge(upload::make_upload_router(
                action_provider.get_image_getter(),
                action_provider.get_image_saver(),
            ))
            .merge(get::make_get_router(action_provider.get_image_getter()))
            .merge(update_canon::make_update_canon_router(
                action_provider.get_image_canon_updater(),
                manager,
            )),
    )
}
