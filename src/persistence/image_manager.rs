use async_trait::async_trait;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::{
    domain::{
        actions::images::{ImageGetter, ImageSaver},
        models::Image,
    },
    persistence::entities::{images, prelude::Images},
};

#[derive(Clone)]
pub struct ImageManager {
    db_conn: DatabaseConnection,
}

impl ImageManager {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }
}

impl From<Image> for images::ActiveModel {
    fn from(value: Image) -> Self {
        Self {
            file_name: ActiveValue::Set(value.file_name),
            width: ActiveValue::Set(value.width as i32),
            height: ActiveValue::Set(value.height as i32),
            ..Default::default()
        }
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
impl ImageGetter for ImageManager {
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

#[async_trait]
impl ImageSaver for ImageManager {
    async fn save_image(&self, image: Image) -> Result<(), Box<dyn std::error::Error>> {
        let model: images::ActiveModel = image.into();
        Images::insert(model).exec(&self.db_conn).await?;

        Ok(())
    }
}
