use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    api::{routing::ApiError, IMAGES_DIR, SCALED_IMAGE_PREFIX},
    domain::{
        actions::image::DeleteImage,
        event_system::{ScreensaverEvent, ScreensaverEventSys},
        screensaver::Screensaver,
    },
};

pub fn make_delete_router(
    di: impl 'static + Clone + Send + Sync + DeleteImage,
    screensaver: impl 'static + Clone + Send + Sync + Screensaver,
    event_mngr: impl 'static + Clone + Send + Sync + ScreensaverEventSys,
) -> Router {
    Router::new().route(
        "/delete",
        post(|body| delete_image(body, di, event_mngr, screensaver)),
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteInput {
    file_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum DeleteImageError {
    Fs(String),
    Persistence(String),
    FailedToDeleteInQueue,
}

impl ApiError for DeleteImageError {}

async fn delete_image(
    Json(input): Json<DeleteInput>,
    di: impl DeleteImage,
    event_mngr: impl ScreensaverEventSys,
    mut screensaver: impl Screensaver,
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

    screensaver.delete_image(&input.file_name).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            DeleteImageError::FailedToDeleteInQueue.to_json_string(),
        )
    })?;

    event_mngr.send(ScreensaverEvent::ScreenSaverUpdated);

    Ok(())
}

async fn delete_fs(input: &DeleteInput) -> Result<(), String> {
    fs::remove_file(format!("{}/{}", IMAGES_DIR, input.file_name))
        .await
        .map_err(|e| e.to_string())?;

    fs::remove_file(format!(
        "{}/{}{}",
        IMAGES_DIR, SCALED_IMAGE_PREFIX, input.file_name
    ))
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}
