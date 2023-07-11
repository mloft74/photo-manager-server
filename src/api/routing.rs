use axum::Router;

use crate::domain::actions::ActionProvider;

mod image;

pub fn make_api_router(action_provider: &(impl ActionProvider + 'static)) -> Router {
    Router::new().nest(
        "/api",
        Router::new().merge(image::make_image_router(action_provider)),
    )
}
