use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{
    domain::models::Image,
    persistence::{
        entities::{images, prelude::Images},
        image::active_model_for_insert_from,
    },
};

#[derive(Clone)]
pub struct ImageSaver {
    db_conn: DatabaseConnection,
}

impl ImageSaver {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }

    pub async fn save_image(&self, image: &Image) -> Result<(), String> {
        let model: images::ActiveModel = active_model_for_insert_from(image);
        Images::insert(model)
            .exec(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
