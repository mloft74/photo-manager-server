use axum::{routing::post, Router};
use hyper::StatusCode;

use crate::{
    api::{
        canon::{self, UpdateCanonError},
        routing::ApiError,
    },
    domain::{actions::image::UpdateCanon, screen_saver_manager::ScreenSaverManager},
};

pub fn make_update_canon_router(
    uc: impl 'static + Clone + Send + Sync + UpdateCanon,
    mngr: ScreenSaverManager,
) -> Router {
    Router::new().route("/update_canon", post(|| update_canon(uc, mngr)))
}

impl ApiError for UpdateCanonError {}

async fn update_canon(
    uc: impl UpdateCanon,
    mut mngr: ScreenSaverManager,
) -> Result<(), (StatusCode, String)> {
    canon::update_canon(&uc, &mut mngr)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_json_string()))?;

    Ok(())
}
