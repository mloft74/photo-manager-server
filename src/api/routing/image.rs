use axum::Router;
use serde::Serialize;

use crate::{
    domain::models::Image, persistence::PersistenceManager,
    state::screensaver_manager::ScreensaverManager,
};

mod current;
mod delete;
mod get;
mod paginated;
mod rename;
mod resolve;
mod update_canon;
mod upload;

pub fn make_image_router(
    persistence_mngr: &PersistenceManager,
    screensaver_mngr: &ScreensaverManager,
) -> Router {
    Router::new().nest(
        "/image",
        Router::new()
            .merge(upload::make_upload_router(
                persistence_mngr.clone(),
                screensaver_mngr.clone(),
            ))
            .merge(get::make_get_router(persistence_mngr.clone()))
            .merge(update_canon::make_update_canon_router(
                persistence_mngr.clone(),
                screensaver_mngr.clone(),
            ))
            .merge(paginated::make_paginated_router(persistence_mngr.clone()))
            .merge(rename::make_rename_router(
                persistence_mngr.clone(),
                screensaver_mngr.clone(),
            ))
            .merge(current::make_current_router(screensaver_mngr.clone()))
            .merge(resolve::make_resolve_router(screensaver_mngr.clone()))
            .merge(delete::make_delete_router(
                persistence_mngr.clone(),
                screensaver_mngr.clone(),
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
