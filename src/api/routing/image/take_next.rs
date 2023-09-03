use axum::{routing::post, Json, Router};
use hyper::StatusCode;
use serde::Serialize;

use crate::{
    api::routing::{image::ImageResponse, ApiError},
    domain::{actions::image::FetchCanon, screensaver::Screensaver},
};

pub fn make_take_next_router(
    mngr: impl 'static + Clone + Send + Sync + Screensaver,
    fc: impl 'static + Clone + Send + Sync + FetchCanon,
) -> Router {
    Router::new()
        // Using post as this route mutates state
        .route("/take_next", post(|| take_next(mngr, fc)))
}

#[derive(Serialize)]
enum TakeNextImageError {
    _FailedToFetchCanon(String),
    _NoDataAfterReloadingImages,
}

impl ApiError for TakeNextImageError {}

async fn take_next(
    mut _mngr: impl Screensaver,
    _fc: impl FetchCanon,
) -> Result<Json<ImageResponse>, (StatusCode, String)> {
    panic!("this method is not and will not be implemented")
    // let image = mngr.take_next();
    // if let Some(image) = image {
    //     Ok(Json(image.into()))
    // } else {
    //     let images = fc.fetch_canon().await.map_err(|e| {
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             TakeNextImageError::FailedToFetchCanon(e).to_json_string(),
    //         )
    //     })?;
    //     mngr.replace(images.into_iter());

    //     let image = mngr.take_next();
    //     if let Some(image) = image {
    //         Ok(Json(image.into()))
    //     } else {
    //         Err((
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             TakeNextImageError::NoDataAfterReloadingImages.to_json_string(),
    //         ))
    //     }
    // }
}
