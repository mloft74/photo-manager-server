use axum::{extract::Query, routing::get, Json, Router};
use hyper::StatusCode;
use serde::Deserialize;

use crate::{api::routing::image::ImageResponse, domain::actions::image::FetchTestImage};

pub fn make_get_router(fi: impl 'static + Clone + Send + Sync + FetchTestImage) -> Router {
    Router::new().route("/get", get(|query| get_image(query, fi)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FindImage {
    file_name: String,
}

async fn get_image(
    Query(find_image): Query<FindImage>,
    fi: impl FetchTestImage,
) -> Result<Json<ImageResponse>, (StatusCode, String)> {
    let file_name = &find_image.file_name;
    let image = fi
        .fetch_image(file_name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Could not find image with file name {}", file_name),
            )
        })?;

    Ok(Json(image.into()))
}
