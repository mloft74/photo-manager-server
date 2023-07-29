use axum::{extract::State, routing::post, Json, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;

use crate::{api::IMAGES_DIR, persistence::image::image_renamer::ImageRenamer};

pub fn make_rename_router(renamer: ImageRenamer) -> Router {
    Router::new()
        .route("/rename", post(rename_image))
        .with_state(renamer)
}

#[derive(Serialize)]
struct RenameImageErrorWrapper<'a> {
    error: &'a RenameImageError,
}

#[derive(Serialize)]
enum RenameImageError {
    FsRenameErr(String),
    PersistenceRenameErr(String),
}

impl RenameImageError {
    fn to_json_string(&self) -> String {
        serde_json::to_string(&RenameImageErrorWrapper { error: self }).unwrap_or_else(|e| {
            json!({
                "error": "jsonConverionFailed",
                "message": e.to_string(),
            })
            .to_string()
        })
    }
}

#[derive(Deserialize)]
struct RenameInput {
    old_name: String,
    new_name: String,
}

async fn rename_image(
    state: State<ImageRenamer>,
    Json(input): Json<RenameInput>,
) -> Result<(), (StatusCode, String)> {
    rename_fs(&input).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            RenameImageError::FsRenameErr(e).to_json_string(),
        )
    })?;

    state
        .rename_image(&input.old_name, &input.new_name)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                RenameImageError::PersistenceRenameErr(e).to_json_string(),
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
