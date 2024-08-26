use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::domain::screensaver::Screensaver;

pub fn make_line_up_router(
    screensaver_mngr: impl 'static + Clone + Send + Sync + Screensaver,
) -> Router {
    Router::new().route("/line_up", get(|| async { line_up(screensaver_mngr) }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LineUpResponse {}

fn line_up(screensaver: impl Screensaver) -> Json<LineUpResponse> {
    todo!();
}
