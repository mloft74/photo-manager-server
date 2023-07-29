use axum::{extract::State, routing::post, Json, Router};
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::persistence::image::image_renamer::ImageRenamer;

pub fn make_rename_router(renamer: ImageRenamer) -> Router {
    Router::new()
        .route("/rename", post(rename_image))
        .with_state(renamer)
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
    state
        .rename_image(&input.old_name, &input.new_name)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"message": e}).to_string(),
            )
        })?;

    Ok(())
}
