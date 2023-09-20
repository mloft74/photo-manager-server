use axum::{routing::post, Router};
use hyper::StatusCode;

use crate::{
    api::{
        canon::{self, UpdateCanonError},
        routing::ApiError,
    },
    domain::{actions::image::UpdateCanon, screensaver::Screensaver},
};

pub fn make_update_canon_router(
    uc: impl 'static + Clone + Send + Sync + UpdateCanon,
    screensaver: impl 'static + Clone + Send + Sync + Screensaver,
) -> Router {
    Router::new().route("/update_canon", post(|| update_canon(uc, screensaver)))
}

impl ApiError for UpdateCanonError {}

async fn update_canon(
    uc: impl UpdateCanon,
    mut screensaver: impl Screensaver,
) -> Result<(), (StatusCode, String)> {
    canon::update_canon(&uc, &mut screensaver)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_json_string()))?;

    Ok(())
}
