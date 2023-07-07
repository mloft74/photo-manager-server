use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

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

#[async_trait]
impl ImageGetter for ImageManager {
    async fn get_image(&self, file_name: &str) -> Result<Image, Box<dyn std::error::Error>> {
        let model = Images::find()
            .filter(images::Column::FileName.eq(file_name))
            .one(&self.db_conn)
            .await?
            .ok_or_else(|| format!("Could not find image with file name {}", file_name))?;

        Ok(Image {
            file_name: model.file_name,
        })
    }
}

#[async_trait]
impl ImageSaver for ImageManager {
    async fn save_image(&self, image: &Image) -> Result<(), Box<dyn std::error::Error>> {
        let model = images::ActiveModel {
            file_name: sea_orm::ActiveValue::Set(image.file_name.clone()),
            ..Default::default()
        };
        Images::insert(model).exec(&self.db_conn).await?;

        Ok(())
    }
}
