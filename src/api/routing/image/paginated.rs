use axum::{extract::Query, routing::get, Json, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{api::routing::image::ImageResponse, domain::actions::image::FetchImagesPage};

pub fn make_paginated_router(
    fetch_images_page_op: impl 'static + Clone + Send + Sync + FetchImagesPage,
) -> Router {
    Router::new().route(
        "/paginated",
        get(|query| get_images(query, fetch_images_page_op)),
    )
}

#[derive(Deserialize)]
struct ImagesPageInput {
    count: u64,
    after: Option<i32>,
}

#[derive(Serialize)]
struct ImagesPageResponse {
    images: Vec<ImageResponse>,
    cursor: Option<i32>,
}

async fn get_images(
    Query(input): Query<ImagesPageInput>,
    fetch_images_page_op: impl FetchImagesPage,
) -> Result<Json<ImagesPageResponse>, (StatusCode, String)> {
    fetch_images_page_op
        .fetch_images_page(input.count, input.after)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
        .map(|v| {
            Json(ImagesPageResponse {
                cursor: v.cursor,
                images: v.images.into_iter().map(|v| v.into()).collect(),
            })
        })
}
