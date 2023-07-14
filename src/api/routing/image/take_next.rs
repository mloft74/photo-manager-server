use axum::{extract::State, routing::post, Json, Router};
use hyper::StatusCode;

use crate::{api::routing::image::ImageResponse, domain::screen_saver_manager::ScreenSaverManager};

pub fn make_take_next_router(manager: &ScreenSaverManager) -> Router {
    Router::new()
        // Using post as this route mutates state
        .route("/take_next", post(take_next))
        .with_state(manager.clone())
}

async fn take_next(
    mut state: State<ScreenSaverManager>,
) -> Result<Json<ImageResponse>, (StatusCode, String)> {
    let image = state.take_next();
    if let Some(image) = image {
        Ok(Json(image.into()))
    } else {
        Err((StatusCode::INTERNAL_SERVER_ERROR, "foo".to_string()))
    }
}
