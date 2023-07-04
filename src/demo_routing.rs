use axum::{extract::State, routing::post, Router};
use deadpool_diesel::postgres::Pool;

pub fn get_demo_router(pool: &Pool) -> Router {
    Router::new()
        .route("/image", post(post_image))
        .with_state(pool.clone())
}

async fn post_image(state: State<Pool>) {}
