use axum::{http::StatusCode, routing::post, Router};

use crate::{
    api::{
        canon::{self, UpdateCanonError},
        routing::ApiError,
    },
    domain::{
        actions::image::UpdateCanon,
        event_system::{ScreensaverEvent, ScreensaverEventSys},
        screensaver::Screensaver,
    },
};

pub fn make_update_canon_router(
    uc: impl 'static + Clone + Send + Sync + UpdateCanon,
    screensaver: impl 'static + Clone + Send + Sync + Screensaver,
    event_mngr: impl 'static + Clone + Send + Sync + ScreensaverEventSys,
) -> Router {
    Router::new().route(
        "/update_canon",
        post(|| update_canon(uc, event_mngr, screensaver)),
    )
}

impl ApiError for UpdateCanonError {}

async fn update_canon(
    uc: impl UpdateCanon,
    event_mngr: impl ScreensaverEventSys,
    mut screensaver: impl Screensaver,
) -> Result<(), (StatusCode, String)> {
    canon::update_canon(&uc, &mut screensaver)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_json_string()))?;

    event_mngr.send(ScreensaverEvent::ScreenSaverUpdated);

    Ok(())
}
