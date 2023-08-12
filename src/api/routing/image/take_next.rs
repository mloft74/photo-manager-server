use axum::{routing::post, Json, Router};
use hyper::StatusCode;
use serde::Serialize;

use crate::{
    api::routing::{image::ImageResponse, ApiError},
    domain::{actions::image::FetchCanon, screen_saver_manager::ScreenSaverManager},
};

pub fn make_take_next_router(
    mngr: ScreenSaverManager,
    fetch_canon_op: impl 'static + Clone + Send + Sync + FetchCanon,
) -> Router {
    Router::new()
        // Using post as this route mutates state
        .route("/take_next", post(|| take_next(mngr, fetch_canon_op)))
}

#[derive(Serialize)]
enum TakeNextImageError {
    FailedToFetchCanon(String),
    NoDataAfterReloadingImages,
}

impl ApiError for TakeNextImageError {}

async fn take_next(
    mut mngr: ScreenSaverManager,
    fetch_canon_op: impl FetchCanon,
) -> Result<Json<ImageResponse>, (StatusCode, String)> {
    let image = mngr.take_next();
    if let Some(image) = image {
        Ok(Json(image.into()))
    } else {
        let images = fetch_canon_op.fetch_canon().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                TakeNextImageError::FailedToFetchCanon(e).to_json_string(),
            )
        })?;
        mngr.replace(images.into_iter());

        let image = mngr.take_next();
        if let Some(image) = image {
            Ok(Json(image.into()))
        } else {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                TakeNextImageError::NoDataAfterReloadingImages.to_json_string(),
            ))
        }
    }
}
