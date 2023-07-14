use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use serde::Deserialize;

use crate::{api::routing::image::ImageResponse, domain::actions::images::ImageGetter};

pub fn make_get_router<T: ImageGetter + 'static>(image_getter: T) -> Router {
    Router::new()
        .route("/get", get(get_image::<T>))
        .with_state(image_getter)
}

#[derive(Deserialize)]
struct FindImage {
    file_name: String,
}

async fn get_image<T: ImageGetter>(
    state: State<T>,
    Query(find_image): Query<FindImage>,
) -> Result<Json<ImageResponse>, (StatusCode, String)> {
    let file_name = &find_image.file_name;
    let image = state
        .get_image(file_name)
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
