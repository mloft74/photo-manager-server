use std::fmt::Display;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde_json::json;

#[derive(Debug)]
pub struct AppError(pub StatusCode, pub Box<dyn std::error::Error>);

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error occurred while processing this request: {}",
            self.1
        )
    }
}

impl std::error::Error for AppError {}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let AppError(status, error_message) = self;
        let body = Json(json!({
            "error": error_message.to_string(),
        }));

        (status, body).into_response()
    }
}
