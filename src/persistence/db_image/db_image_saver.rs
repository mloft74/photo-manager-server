use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{
    domain::{actions::images::ImageSaver, models::Image},
    persistence::{
        db_image::active_model_for_insert_from,
        entities::{images, prelude::Images},
    },
};

#[derive(Clone)]
pub struct DbImageSaver {
    db_conn: DatabaseConnection,
}

impl DbImageSaver {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }
}

#[async_trait]
impl ImageSaver for DbImageSaver {
    async fn save_image(&self, image: &Image) -> Result<(), Box<dyn std::error::Error>> {
        let model: images::ActiveModel = active_model_for_insert_from(image);
        Images::insert(model).exec(&self.db_conn).await?;

        Ok(())
    }
}
