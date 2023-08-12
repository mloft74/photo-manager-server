use async_trait::async_trait;
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    domain::actions::image::RenameImage,
    persistence::{
        entities::{images, prelude::Images},
        persistence_manager::PersistenceManager,
    },
};

#[async_trait]
impl RenameImage for PersistenceManager {
    async fn rename_image(&self, old_name: &str, new_name: &str) -> Result<(), String> {
        Images::update_many()
            .col_expr(images::Column::FileName, Expr::value(new_name))
            .filter(images::Column::FileName.eq(old_name))
            .exec(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
