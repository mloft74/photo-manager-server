use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{api::error_handling::AppError, domain::actions::images::ImageGetter};

pub fn make_get_router<T: ImageGetter + 'static>(image_getter: T) -> Router {
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
}

async fn get_image<T: ImageGetter>(
    state: State<T>,
    Query(find_image): Query<FindImage>,
) -> Result<Json<ImageResponse>, AppError> {
    let file_name = &find_image.file_name;
    let image = state.get_image(file_name).await?.ok_or_else(|| {
        AppError(
            StatusCode::NOT_FOUND,
            format!("Could not find image with file name {}", file_name).into(),
        )
    })?;

    Ok(Json(ImageResponse {
        file_name: image.file_name,
    }))
}
