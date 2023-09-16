use axum::{routing::post, Json, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    api::{routing::ApiError, IMAGES_DIR},
    domain::{actions::image::DeleteImage, screensaver::Screensaver},
};

pub fn make_delete_router(
    di: impl 'static + Clone + Send + Sync + DeleteImage,
    ss_mngr: impl 'static + Clone + Send + Sync + Screensaver,
) -> Router {
    Router::new().route("/delete", post(|body| delete_image(body, di, ss_mngr)))
}

#[derive(Deserialize)]
struct DeleteInput {
    file_name: String,
}

#[derive(Serialize)]
enum DeleteImageError {
    Fs(String),
    Persistence(String),
    FailedToDeleteInQueue,
}

impl ApiError for DeleteImageError {}

async fn delete_image(
    Json(input): Json<DeleteInput>,
    di: impl DeleteImage,
    mut ss_mngr: impl Screensaver,
) -> Result<(), (StatusCode, String)> {
    delete_fs(&input).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            DeleteImageError::Fs(e).to_json_string(),
        )
    })?;

    di.delete_image(&input.file_name).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            DeleteImageError::Persistence(e).to_json_string(),
        )
    })?;

    ss_mngr.delete_image(&input.file_name).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            DeleteImageError::FailedToDeleteInQueue.to_json_string(),
        )
    })?;

    Ok(())
}

async fn delete_fs(input: &DeleteInput) -> Result<(), String> {
    fs::remove_file(format!("{}/{}", IMAGES_DIR, input.file_name))
        .await
        .map_err(|e| e.to_string())
}
