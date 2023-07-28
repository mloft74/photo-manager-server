use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    api::routing::image::ImageResponse,
    persistence::image::paginated_images_fetcher::PaginatedImagesFetcher,
};

pub fn make_paginated_router(images_fetcher: PaginatedImagesFetcher) -> Router {
    Router::new()
        .route("/paginated", get(get_images))
        .with_state(images_fetcher)
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
    state: State<PaginatedImagesFetcher>,
    Query(input): Query<ImagesPageInput>,
) -> Result<Json<ImagesPageResponse>, (StatusCode, String)> {
    state
        .fetch_images(input.count, input.after)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
        .map(|v| {
            Json(ImagesPageResponse {
                cursor: v.cursor,
                images: v.images.into_iter().map(|v| v.into()).collect(),
            })
        })
}
