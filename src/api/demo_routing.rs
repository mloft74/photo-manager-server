use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    api::error_handling::AppError,
    domain::{
        actions::{
            images::{ImageGetter, ImageSaver},
            ActionProvider,
        },
        models::Image,
    },
};

pub fn make_demo_router(action_provider: &(impl ActionProvider + 'static)) -> Router {
    Router::new()
        .merge(make_add_image_router(action_provider.get_image_saver()))
        .merge(make_get_image_router(action_provider.get_image_getter()))
}

fn make_add_image_router<T: ImageSaver + 'static>(image_saver: T) -> Router {
    Router::new()
        .route("/add_image", post(post_image::<T>))
        .with_state(image_saver)
}

fn make_get_image_router<T: ImageGetter + 'static>(image_getter: T) -> Router {
    Router::new()
        .route("/get_image", get(get_image::<T>))
        .with_state(image_getter)
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
