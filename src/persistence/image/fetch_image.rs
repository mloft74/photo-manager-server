use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    domain::{actions::image::FetchImage, models::Image},
    persistence::{
        entities::{images, prelude::Images},
        PersistenceManager,
    },
};

#[async_trait]
impl FetchImage for PersistenceManager {
    async fn fetch_image(&self, file_name: &str) -> Result<Option<Image>, String> {
        Ok(Images::find()
            .filter(images::Column::FileName.eq(file_name))
            .one(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?
            .map(|m| m.into()))
    }
}
