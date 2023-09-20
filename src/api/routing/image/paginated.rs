use axum::{extract::Query, routing::get, Json, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    api::routing::image::ImageResponse,
    domain::actions::image::{FetchImagesPage, PaginationOrder},
};

pub fn make_paginated_router(fip: impl 'static + Clone + Send + Sync + FetchImagesPage) -> Router {
    Router::new().route("/paginated", get(|query| get_images(query, fip)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImagesPageInput {
    count: u64,
    after: Option<i32>,
    order: Option<InputOrder>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum InputOrder {
    NewToOld,
    OldToNew,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImagesPageResponse {
    images: Vec<ImageResponse>,
    cursor: Option<i32>,
}

async fn get_images(
    Query(input): Query<ImagesPageInput>,
    fip: impl FetchImagesPage,
) -> Result<Json<ImagesPageResponse>, (StatusCode, String)> {
    fip.fetch_images_page(
        input.count,
        input.after,
        match input.order {
            Some(InputOrder::NewToOld) => PaginationOrder::NewToOld,
            Some(InputOrder::OldToNew) => PaginationOrder::OldToNew,
            None => PaginationOrder::OldToNew,
        },
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
    .map(|v| {
        Json(ImagesPageResponse {
            cursor: v.cursor,
            images: v.images.into_iter().map(|v| v.into()).collect(),
        })
    })
}
