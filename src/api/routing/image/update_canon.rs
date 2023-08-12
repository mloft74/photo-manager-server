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
    canon_updater: impl 'static + Clone + Send + Sync + UpdateCanon,
    mngr: ScreenSaverManager,
) -> Router {
    Router::new().route("/update_canon", post(|| update_canon(canon_updater, mngr)))
}

impl ApiError for UpdateCanonError {}

async fn update_canon(
    update_canon_op: impl UpdateCanon,
    mut mngr: ScreenSaverManager,
) -> Result<(), (StatusCode, String)> {
    canon::update_canon(&update_canon_op, &mut mngr)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_json_string()))?;

    Ok(())
}
