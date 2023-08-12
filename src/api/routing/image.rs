use axum::Router;
use serde::Serialize;

use crate::{
    domain::{models::Image, screen_saver_manager::ScreenSaverManager},
    persistence::persistence_manager::PersistenceManager,
};

mod delete;
mod get;
mod paginated;
mod rename;
mod take_next;
mod update_canon;
mod upload;

pub fn make_image_router(
    persistence_mngr: &PersistenceManager,
    ss_mngr: &ScreenSaverManager,
) -> Router {
    Router::new().nest(
        "/image",
        Router::new()
            .merge(upload::make_upload_router(
                persistence_mngr.clone(),
                ss_mngr.clone(),
            ))
            .merge(get::make_get_router(persistence_mngr.clone()))
            .merge(update_canon::make_update_canon_router(
                persistence_mngr.clone(),
                ss_mngr.clone(),
            ))
            .merge(take_next::make_take_next_router(
                ss_mngr.clone(),
                persistence_mngr.clone(),
            ))
            .merge(paginated::make_paginated_router(persistence_mngr.clone()))
            .merge(rename::make_rename_router(persistence_mngr.clone()))
            .merge(delete::make_delete_router(persistence_mngr.clone())),
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
