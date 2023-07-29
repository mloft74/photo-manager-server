use axum::{extract::State, routing::post, Json, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    api::{routing::ApiError, IMAGES_DIR},
    persistence::image::image_deleter::ImageDeleter,
};

pub fn make_delete_router(image_deleter: ImageDeleter) -> Router {
    Router::new()
        .route("/delete", post(delete_image))
        .with_state(image_deleter)
}

#[derive(Deserialize)]
struct DeleteInput {
    file_name: String,
}

#[derive(Serialize)]
enum DeleteImageError {
    Fs(String),
    Persistence(String),
}

impl ApiError for DeleteImageError {}

async fn delete_image(
    state: State<ImageDeleter>,
    Json(input): Json<DeleteInput>,
) -> Result<(), (StatusCode, String)> {
    delete_fs(&input).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            DeleteImageError::Fs(e).to_json_string(),
        )
    })?;

    state.delete_image(&input.file_name).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            DeleteImageError::Persistence(e).to_json_string(),
        )
    })?;

    Ok(())
}

async fn delete_fs(input: &DeleteInput) -> Result<(), String> {
    fs::remove_file(format!("{}/{}", IMAGES_DIR, input.file_name))
        .await
        .map_err(|e| e.to_string())
}
