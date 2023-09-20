use axum::{routing::get, Json, Router};
use serde::Serialize;

pub fn make_ping_router() -> Router {
    Router::new().route("/ping", get(ping))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PingResponse {
    message: String,
}

async fn ping() -> Json<PingResponse> {
    Json(PingResponse {
        message: "pong".to_string(),
    })
}
