use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    domain::actions::image::DeleteImage,
    persistence::{
        entities::{images, prelude::Images},
        PersistenceManager,
    },
};

#[async_trait]
impl DeleteImage for PersistenceManager {
    async fn delete_image(&self, file_name: &str) -> Result<(), String> {
        Images::delete_many()
            .filter(images::Column::FileName.eq(file_name))
            .exec(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
