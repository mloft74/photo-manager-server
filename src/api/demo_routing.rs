use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;

use crate::{
    api::error_handling::AppError,
    domain::{actions::images::ImageSaver, models::Image},
};

pub fn make_demo_router<T: ImageSaver + 'static>(image_manager: T) -> Router {
    Router::new()
        .route("/add_image", post(post_image::<T>))
        .with_state(image_manager)
}

#[derive(Deserialize)]
struct NewImage {
    file_name: String,
}

async fn post_image<T: ImageSaver>(
    state: State<T>,
    Json(new_image): Json<NewImage>,
) -> Result<(), AppError> {
    Ok(state
        .save_image(&Image {
            file_name: new_image.file_name,
        })
        .await?)
}
