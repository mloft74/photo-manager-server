use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::{api::routing::image::ImageResponse, domain::screensaver::Screensaver};

pub fn make_current_router(
    screensaver: impl 'static + Clone + Send + Sync + Screensaver,
) -> Router {
    Router::new().route("/current", get(|| async { current(screensaver) }))
}

#[derive(Serialize)]
struct CurrentResponse {
    image: Option<ImageResponse>,
}

fn current(screensaver: impl Screensaver) -> Json<CurrentResponse> {
    Json(CurrentResponse {
        image: screensaver.current().map(|i| i.into()),
    })
}
