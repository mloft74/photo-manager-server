use axum::{extract::State, routing::post, Json, Router};
use deadpool_diesel::postgres::Pool;
use diesel::RunQueryDsl;
use serde::Deserialize;

use crate::{api::error_handling::AppError, database::models::Image, schema};

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
