use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::{
    domain::{actions::images::ImageGetter, models::Image},
    persistence::entities::{images, prelude::Images},
};

#[derive(Clone)]
pub struct DbImageGetter {
    db_conn: DatabaseConnection,
}

impl DbImageGetter {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }
}

impl From<images::Model> for Image {
    fn from(value: images::Model) -> Self {
        Self {
            file_name: value.file_name,
            width: value.width as u32,
            height: value.height as u32,
        }
    }
}

#[async_trait]
impl ImageGetter for DbImageGetter {
    async fn get_image(
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
