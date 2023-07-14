use axum::Router;
use serde::Serialize;

use crate::domain::{
    actions::ActionProvider, models::Image, screen_saver_manager::ScreenSaverManager,
};

mod get;
mod take_next;
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
                action_provider.get_image_fetcher(),
                action_provider.get_image_saver(),
                manager,
            ))
            .merge(get::make_get_router(action_provider.get_image_fetcher()))
            .merge(update_canon::make_update_canon_router(
                action_provider.get_image_canon_updater(),
                manager,
            ))
            .merge(take_next::make_take_next_router(
                action_provider.get_image_canon_fetcher(),
                manager,
            )),
    )
}

#[derive(Serialize)]
struct ImageResponse {
    file_name: String,
    width: u32,
    height: u32,
}

impl From<Image> for ImageResponse {
    fn from(value: Image) -> Self {
        Self {
            file_name: value.file_name,
            width: value.width,
            height: value.height,
        }
    }
}
