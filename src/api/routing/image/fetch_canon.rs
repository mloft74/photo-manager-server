use axum::{Json, Router, routing::get};
use hyper::StatusCode;
use serde::Serialize;

use crate::{api::routing::image::ImageResponse, domain::actions::image::FetchCanon};

pub fn make_fetch_canon_router(fc: impl 'static + Clone + Send + Sync + FetchCanon) -> Router {
    Router::new().route("/fetch_canon", get(|| fetch_canon(fc)))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchCanonResponse {
    images: Vec<ImageResponse>,
}

async fn fetch_canon(
    fc: impl FetchCanon,
) -> Result<Json<FetchCanonResponse>, (StatusCode, String)> {
    fc.fetch_canon()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
        .map(|v| {
            Json(FetchCanonResponse {
                images: v.into_iter().map(|v| v.into()).collect(),
            })
        })
}
