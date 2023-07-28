use async_trait::async_trait;
use sea_orm::{CursorTrait, DatabaseConnection, EntityTrait};

use crate::{
    domain::{actions::images::PaginatedImagesFetcher, models::Image},
    persistence::entities::{images, prelude::Images},
};

#[derive(Clone)]
pub struct DbPaginatedImagesFetcher {
    db_conn: DatabaseConnection,
}

impl DbPaginatedImagesFetcher {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }
}

#[async_trait]
impl PaginatedImagesFetcher for DbPaginatedImagesFetcher {
    async fn fetch_images(&self, count: u64, after: Option<i32>) -> Result<Vec<Image>, String> {
        let mut pagination = Images::find().cursor_by(images::Column::Id);
        let pagination = match after {
            Some(cursor) => pagination.after(cursor),
            None => &mut pagination,
        };
        let images: Vec<Image> = pagination
            .first(count)
            .all(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|m| m.into())
            .collect();

        Ok(images)
    }
}
