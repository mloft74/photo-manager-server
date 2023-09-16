use axum::{routing::get, Json, Router};
use hyper::StatusCode;

use crate::{api::routing::image::ImageResponse, domain::screensaver::Screensaver};

pub fn make_current_router(mngr: impl 'static + Clone + Send + Sync + Screensaver) -> Router {
    Router::new().route("/current", get(|| async { current(mngr) }))
}

fn current(mngr: impl Screensaver) -> Result<Json<ImageResponse>, (StatusCode, String)> {
    let current = mngr.current().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "no current image".to_string(),
    ))?;
    Ok(Json(current.into()))
}
