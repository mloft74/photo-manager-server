use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use serde::Deserialize;

use crate::{api::routing::image::ImageResponse, domain::actions::images::ImageFetcher};

pub fn make_get_router<T: ImageFetcher + 'static>(image_fetcher: T) -> Router {
    Router::new()
        .route("/get", get(get_image::<T>))
        .with_state(image_fetcher)
}

#[derive(Deserialize)]
struct FindImage {
    file_name: String,
}

async fn get_image<T: ImageFetcher>(
    state: State<T>,
    Query(find_image): Query<FindImage>,
) -> Result<Json<ImageResponse>, (StatusCode, String)> {
    let file_name = &find_image.file_name;
    let image = state
        .fetch_image(file_name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Could not find image with file name {}", file_name),
            )
        })?;

    Ok(Json(image.into()))
}
