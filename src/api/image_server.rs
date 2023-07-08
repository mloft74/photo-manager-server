use axum::Router;
use tower_http::services::ServeDir;

use crate::api::IMAGES_DIR;

pub fn create_image_server_router() -> Router {
    Router::new().nest_service("/image", ServeDir::new(IMAGES_DIR))
}
