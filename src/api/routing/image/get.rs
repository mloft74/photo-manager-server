use axum::{extract::Query, routing::get, Json, Router};
use hyper::StatusCode;
use serde::Deserialize;

use crate::{api::routing::image::ImageResponse, domain::actions::image::FetchImage};

pub fn make_get_router(fetch_image_op: impl 'static + Clone + Send + Sync + FetchImage) -> Router {
    Router::new().route("/get", get(|query| get_image(query, fetch_image_op)))
}

#[derive(Deserialize)]
struct FindImage {
    file_name: String,
}

async fn get_image(
    Query(find_image): Query<FindImage>,
    fetch_image_op: impl FetchImage,
) -> Result<Json<ImageResponse>, (StatusCode, String)> {
    let file_name = &find_image.file_name;
    let image = fetch_image_op
        .fetch_image(file_name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Could not find image with file name {}", file_name),
            )
        })?;

    Ok(Json(image.into()))
}
