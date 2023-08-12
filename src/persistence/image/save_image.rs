use async_trait::async_trait;
use sea_orm::EntityTrait;

use crate::{
    domain::{actions::image::SaveImage, models::Image},
    persistence::{
        entities::{images, prelude::Images},
        image::active_model_for_insert_from,
        persistence_manager::PersistenceManager,
    },
};

#[async_trait]
impl SaveImage for PersistenceManager {
    async fn save_image(&self, image: &Image) -> Result<(), String> {
        let model: images::ActiveModel = active_model_for_insert_from(image);
        Images::insert(model)
            .exec(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
