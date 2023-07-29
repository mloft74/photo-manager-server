use axum::{extract::State, routing::post, Json, Router};
use hyper::StatusCode;
use serde::Serialize;

use crate::{
    api::routing::{image::ImageResponse, ApiError},
    domain::screen_saver_manager::ScreenSaverManager,
    persistence::image::image_canon_fetcher::ImageCanonFetcher,
};

pub fn make_take_next_router(
    canon_fetcher: ImageCanonFetcher,
    manager: ScreenSaverManager,
) -> Router {
    Router::new()
        // Using post as this route mutates state
        .route("/take_next", post(take_next))
        .with_state(TakeNextState {
            canon_fetcher,
            manager,
        })
}

#[derive(Serialize)]
enum TakeNextImageError {
    FailedToFetchCanon(String),
    NoDataAfterReloadingImages,
}

impl ApiError for TakeNextImageError {}

#[derive(Clone)]
struct TakeNextState {
    canon_fetcher: ImageCanonFetcher,
    manager: ScreenSaverManager,
}

async fn take_next(
    mut state: State<TakeNextState>,
) -> Result<Json<ImageResponse>, (StatusCode, String)> {
    let image = state.manager.take_next();
    if let Some(image) = image {
        Ok(Json(image.into()))
    } else {
        let images = state.canon_fetcher.fetch_canon().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                TakeNextImageError::FailedToFetchCanon(e).to_json_string(),
            )
        })?;
        state.manager.replace(images.into_iter());

        let image = state.manager.take_next();
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
