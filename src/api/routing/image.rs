use axum::Router;

use crate::domain::actions::ActionProvider;

mod add;
mod get;

pub fn make_image_router(action_provider: &(impl ActionProvider + 'static)) -> Router {
    Router::new().nest(
        "/image",
        Router::new()
            .merge(add::make_add_router(action_provider.get_image_saver()))
            .merge(get::make_get_router(action_provider.get_image_getter())),
    )
}
