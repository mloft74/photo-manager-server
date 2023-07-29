use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};

use crate::persistence::entities::{images, prelude::Images};

#[derive(Clone)]
pub struct ImageDeleter {
    db_conn: DbConn,
}

impl ImageDeleter {
    pub(in crate::persistence) fn new(db_conn: DbConn) -> Self {
        Self { db_conn }
    }

    pub async fn delete_image(&self, file_name: &str) -> Result<(), String> {
        Images::delete_many()
            .filter(images::Column::FileName.eq(file_name))
            .exec(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
