use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::{api::routing::image::ImageResponse, domain::screensaver::TestScreensaver};

pub fn make_current_router(
    screensaver: impl 'static + Clone + Send + Sync + TestScreensaver,
) -> Router {
    Router::new().route("/current", get(|| async { current(screensaver) }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CurrentResponse {
    image: Option<ImageResponse>,
}

fn current(screensaver: impl TestScreensaver) -> Json<CurrentResponse> {
    Json(CurrentResponse {
        image: Some(screensaver.current().into()),
    })
}
