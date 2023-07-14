use axum::{extract::State, routing::post, Router};
use hyper::StatusCode;
use serde::Serialize;
use serde_json::json;

use crate::{
    api::canon::{self, UpdateCanonError},
    domain::{actions::images::ImageCanonUpdater, screen_saver_manager::ScreenSaverManager},
};

pub fn make_update_canon_router<T: ImageCanonUpdater + 'static>(
    canon_updater: T,
    manager: &ScreenSaverManager,
) -> Router {
    Router::new()
        .route("/update_canon", post(update_canon))
        .with_state(UpdateCanonState {
            canon_updater,
            manager: manager.clone(),
        })
}

#[derive(Clone)]
struct UpdateCanonState<T: ImageCanonUpdater> {
    canon_updater: T,
    manager: ScreenSaverManager,
}

impl UpdateCanonError {
    fn to_json_string(&self) -> String {
        serde_json::to_string(&UpdateCanonErrorWrapper { error: self }).unwrap_or_else(|e| {
            json!({
                "error": "jsonConverionFailed",
                "message": e.to_string(),
            })
            .to_string()
        })
    }
}

#[derive(Serialize)]
struct UpdateCanonErrorWrapper<'a> {
    error: &'a UpdateCanonError,
}

async fn update_canon<T: ImageCanonUpdater>(
    state: State<UpdateCanonState<T>>,
) -> Result<(), (StatusCode, String)> {
    let UpdateCanonState {
        canon_updater,
        mut manager,
    } = state.0;
    canon::update_canon(&canon_updater, &mut manager)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_json_string()))?;

    Ok(())
}
