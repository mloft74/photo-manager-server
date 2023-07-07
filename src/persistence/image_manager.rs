use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{
    domain::models::Image,
    persistence::entities::{images, prelude::Images},
};

#[derive(Clone)]
pub struct ImageManager {
    db_conn: DatabaseConnection,
}

// Hide this behind Traits and use generics to pass it around.
impl ImageManager {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }

    pub async fn get_image() -> Result<Image, Box<dyn std::error::Error>> {
        todo!()
    }

    pub async fn save_image(&self, image: &Image) -> Result<(), Box<dyn std::error::Error>> {
        let model = images::ActiveModel {
            file_name: sea_orm::ActiveValue::Set(image.file_name.clone()),
            ..Default::default()
        };

        Images::insert(model).exec(&self.db_conn).await?;

        Ok(())
    }
}
