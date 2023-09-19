use axum::{routing::get, Json, Router};
use hyper::StatusCode;

use crate::{api::routing::image::ImageResponse, domain::screensaver::Screensaver};

pub fn make_current_router(
    screensaver: impl 'static + Clone + Send + Sync + Screensaver,
) -> Router {
    Router::new().route("/current", get(|| async { current(screensaver) }))
}

fn current(screensaver: impl Screensaver) -> Result<Json<ImageResponse>, (StatusCode, String)> {
    let current = screensaver.current().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "no current image".to_string(),
    ))?;
    Ok(Json(current.into()))
}
