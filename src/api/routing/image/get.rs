use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::domain::{models::Image, repos::images::ImageRepo};

pub fn make_get_router<T: ImageRepo + 'static>(image_getter: T) -> Router {
    Router::new()
        .route("/get", get(get_image::<T>))
        .with_state(image_getter)
}

#[derive(Deserialize)]
struct FindImage {
    file_name: String,
}

#[derive(Serialize)]
struct ImageResponse {
    file_name: String,
    width: u32,
    height: u32,
}

impl From<Image> for ImageResponse {
    fn from(value: Image) -> Self {
        Self {
            file_name: value.file_name,
            width: value.width,
            height: value.height,
        }
    }
}

async fn get_image<T: ImageRepo>(
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
