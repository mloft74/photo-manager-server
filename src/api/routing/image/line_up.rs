use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::{api::routing::image::ImageResponse, domain::screensaver::Screensaver};

pub fn make_line_up_router(
    screensaver_mngr: impl 'static + Clone + Send + Sync + Screensaver,
) -> Router {
    Router::new().route("/line_up", get(|| async { line_up(screensaver_mngr) }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LineUpResponse {
    images: Vec<ImageResponse>,
    current_idx: Option<usize>,
}

fn line_up(screensaver: impl Screensaver) -> Json<LineUpResponse> {
    let line_up = screensaver.get_line_up();
    Json(LineUpResponse {
        images: line_up.images.into_iter().map(|i| i.into()).collect(),
        current_idx: line_up.current_idx,
    })
}
