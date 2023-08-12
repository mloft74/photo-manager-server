use axum::Router;
use serde::Serialize;
use serde_json::json;

use crate::{
    domain::screen_saver_manager::ScreenSaverManager,
    persistence::persistence_manager::PersistenceManager,
};

mod image;
mod ping;

pub fn make_api_router(
    persistence_mngr: &PersistenceManager,
    ss_mngr: &ScreenSaverManager,
) -> Router {
    Router::new().nest(
        "/api",
        Router::new()
            .merge(image::make_image_router(persistence_mngr, ss_mngr))
            .merge(ping::make_ping_router()),
    )
}

#[derive(Serialize)]
struct ApiErrorWrapper<'a, T: ApiError> {
    error: &'a T,
}

trait ApiError: Serialize {
    fn to_json_string(&self) -> String
    where
        Self: Sized,
    {
        serde_json::to_string(&ApiErrorWrapper { error: self }).unwrap_or_else(|e| {
            json!({
                "error": "jsonConverionFailed",
                "message": e.to_string(),
            })
            .to_string()
        })
    }
}
