use std::fmt::Display;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use deadpool_diesel::{postgres::Pool, InteractError, PoolError};
use diesel::RunQueryDsl;
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::{database::models::Image, schema};

pub fn make_demo_router(pool: &Pool) -> Router {
    Router::new()
        .route("/add_image", post(post_image))
        .with_state(pool.clone())
}

#[derive(Deserialize)]
struct NewImage {
    path: String,
}

async fn post_image(state: State<Pool>, Json(new_image): Json<NewImage>) -> Result<(), AppError> {
    let connection = state.get().await?;
    let rows_affected = connection
        .interact(|conn| {
            diesel::insert_into(schema::images::table)
                .values(Image {
                    path: new_image.path,
                })
                .execute(conn)
        })
        .await??;
    tracing::debug!("post_image affected {} rows", rows_affected);

    Ok(())
}

#[derive(Debug)]
struct AppError(pub StatusCode, Box<dyn std::error::Error>);

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

impl From<PoolError> for AppError {
    fn from(value: PoolError) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, Box::new(value))
    }
}

impl From<InteractError> for AppError {
    fn from(value: InteractError) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, Box::new(value))
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(value: diesel::result::Error) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, Box::new(value))
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
