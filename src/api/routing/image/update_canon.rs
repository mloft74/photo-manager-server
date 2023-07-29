use axum::{extract::State, routing::post, Router};
use hyper::StatusCode;

use crate::{
    api::{
        canon::{self, UpdateCanonError},
        routing::ApiError,
    },
    domain::screen_saver_manager::ScreenSaverManager,
    persistence::image::image_canon_updater::ImageCanonUpdater,
};

pub fn make_update_canon_router(
    canon_updater: ImageCanonUpdater,
    manager: ScreenSaverManager,
) -> Router {
    Router::new()
        .route("/update_canon", post(update_canon))
        .with_state(UpdateCanonState {
            canon_updater,
            manager,
        })
}

#[derive(Clone)]
struct UpdateCanonState {
    canon_updater: ImageCanonUpdater,
    manager: ScreenSaverManager,
}

impl ApiError for UpdateCanonError {}

async fn update_canon(state: State<UpdateCanonState>) -> Result<(), (StatusCode, String)> {
    let UpdateCanonState {
        canon_updater,
        mut manager,
    } = state.0;
    canon::update_canon(&canon_updater, &mut manager)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_json_string()))?;

    Ok(())
}
