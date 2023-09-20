use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::domain::screensaver::{ResolveState, Screensaver};

pub fn make_resolve_router(
    screensaver: impl 'static + Clone + Send + Sync + Screensaver,
) -> Router {
    Router::new().route(
        "/resolve",
        post(|body| async { resolve(body, screensaver) }),
    )
}

#[derive(Deserialize)]
struct ResolveInput {
    file_name: String,
}

#[derive(Serialize)]
struct ResolveResponse {
    resolve_status: ResolveStatus,
}

#[derive(Serialize)]
enum ResolveStatus {
    NoImages,
    NotCurrent,
    Resolved,
}

fn resolve(
    Json(input): Json<ResolveInput>,
    mut screensaver: impl Screensaver,
) -> Json<ResolveResponse> {
    let x = screensaver.resolve(&input.file_name);
    Json(ResolveResponse {
        resolve_status: match x {
            ResolveState::NotCurrent => ResolveStatus::NotCurrent,
            ResolveState::Resolved => ResolveStatus::Resolved,
            ResolveState::NoImages => ResolveStatus::NoImages,
        },
    })
}
