use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    api::{routing::ApiError, IMAGES_DIR},
    domain::{actions::image::RenameImage, screensaver::Screensaver},
};

pub fn make_rename_router(
    ri: impl 'static + Clone + Send + Sync + RenameImage,
    screensaver: impl 'static + Clone + Send + Sync + Screensaver,
) -> Router {
    Router::new().route("/rename", post(|body| rename_image(body, ri, screensaver)))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum RenameImageError {
    Fs(String),
    Persistence(String),
    FailedToRenameInQueue,
}

impl ApiError for RenameImageError {}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenameInput {
    old_name: String,
    new_name: String,
}

async fn rename_image(
    Json(input): Json<RenameInput>,
    ri: impl RenameImage,
    mut screensaver: impl Screensaver,
) -> Result<(), (StatusCode, String)> {
    rename_fs(&input).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            RenameImageError::Fs(e).to_json_string(),
        )
    })?;

    ri.rename_image(&input.old_name, &input.new_name)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                RenameImageError::Persistence(e).to_json_string(),
            )
        })?;

    screensaver
        .rename_image(&input.old_name, &input.new_name)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                RenameImageError::FailedToRenameInQueue.to_json_string(),
            )
        })?;

    Ok(())
}

async fn rename_fs(input: &RenameInput) -> Result<(), String> {
    fs::rename(
        format!("{}/{}", IMAGES_DIR, input.old_name),
        format!("{}/{}", IMAGES_DIR, input.new_name),
    )
    .await
    .map_err(|e| e.to_string())
}
