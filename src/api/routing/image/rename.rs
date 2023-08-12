use axum::{routing::post, Json, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    api::{routing::ApiError, IMAGES_DIR},
    domain::actions::image::RenameImage,
};

pub fn make_rename_router(
    ri: impl 'static + Clone + Send + Sync + RenameImage,
) -> Router {
    Router::new().route("/rename", post(|body| rename_image(body, ri)))
}

#[derive(Serialize)]
enum RenameImageError {
    Fs(String),
    Persistence(String),
}

impl ApiError for RenameImageError {}

#[derive(Deserialize)]
struct RenameInput {
    old_name: String,
    new_name: String,
}

async fn rename_image(
    Json(input): Json<RenameInput>,
    ri: impl RenameImage,
) -> Result<(), (StatusCode, String)> {
    rename_fs(&input).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            RenameImageError::Fs(e).to_json_string(),
        )
    })?;

    ri
        .rename_image(&input.old_name, &input.new_name)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                RenameImageError::Persistence(e).to_json_string(),
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
