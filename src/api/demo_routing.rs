use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;

use crate::{
    api::error_handling::AppError, database::image_manager::ImageManager, domain::models::Image,
};

pub fn make_demo_router(image_manager: ImageManager) -> Router {
    Router::new()
        .route("/add_image", post(post_image))
        .with_state(image_manager)
}

#[derive(Deserialize)]
struct NewImage {
    file_name: String,
}

async fn post_image(
    state: State<ImageManager>,
    Json(new_image): Json<NewImage>,
) -> Result<(), AppError> {
    Ok(state
        .save_image(&Image {
            file_name: new_image.file_name,
        })
        .await?)
}
