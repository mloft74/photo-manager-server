use axum::{extract::State, routing::post, Json, Router};
use hyper::StatusCode;
use serde::Serialize;
use serde_json::json;

use crate::{
    api::routing::image::ImageResponse,
    domain::{actions::images::ImageCanonFetcher, screen_saver_manager::ScreenSaverManager},
};

pub fn make_take_next_router<T: ImageCanonFetcher + 'static>(
    canon_fetcher: T,
    manager: &ScreenSaverManager,
) -> Router {
    Router::new()
        // Using post as this route mutates state
        .route("/take_next", post(take_next))
        .with_state(TakeNextState {
            canon_fetcher,
            manager: manager.clone(),
        })
}

#[derive(Serialize)]
struct TakeNextImageErrorWrapper<'a> {
    error: &'a TakeNextImageError,
}

#[derive(Serialize)]
enum TakeNextImageError {
    FailedToFetchCanon(String),
    NoDataAfterReloadingImages,
}

impl TakeNextImageError {
    fn to_json_string(&self) -> String {
        serde_json::to_string(&TakeNextImageErrorWrapper { error: self }).unwrap_or_else(|e| {
            json!({
                "error": "jsonConverionFailed",
                "message": e.to_string(),
            })
            .to_string()
        })
    }
}

#[derive(Clone)]
struct TakeNextState<T: ImageCanonFetcher> {
    canon_fetcher: T,
    manager: ScreenSaverManager,
}

async fn take_next<T: ImageCanonFetcher>(
    mut state: State<TakeNextState<T>>,
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
