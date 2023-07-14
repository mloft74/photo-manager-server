use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::{
    domain::{actions::images::ImageFetcher, models::Image},
    persistence::entities::{images, prelude::Images},
};

#[derive(Clone)]
pub struct DbImageFetcher {
    db_conn: DatabaseConnection,
}

impl DbImageFetcher {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }
}

#[async_trait]
impl ImageFetcher for DbImageFetcher {
    async fn fetch_image(
        &self,
        file_name: &str,
    ) -> Result<Option<Image>, Box<dyn std::error::Error>> {
        Ok(Images::find()
            .filter(images::Column::FileName.eq(file_name))
            .one(&self.db_conn)
            .await?
            .map(|m| m.into()))
    }
}
