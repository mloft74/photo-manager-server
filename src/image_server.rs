use axum::Router;
use tower_http::services::ServeDir;

pub fn create_image_server_router() -> Router {
    Router::new().nest_service(
        "/images",
        ServeDir::new("/var/lib/photo_manager_server/images"),
    )
}
