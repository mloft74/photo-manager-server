use sea_orm::{sea_query::Expr, ColumnTrait, DbConn, EntityTrait, QueryFilter};

use crate::persistence::entities::{images, prelude::Images};

#[derive(Clone)]
pub struct ImageRenamer {
    db_conn: DbConn,
}

impl ImageRenamer {
    pub fn new(db_conn: DbConn) -> Self {
        Self { db_conn }
    }

    pub async fn rename_image(&self, old_name: &str, new_name: &str) -> Result<(), String> {
        Images::update_many()
            .col_expr(images::Column::FileName, Expr::value(new_name))
            .filter(images::Column::FileName.eq(old_name))
            .exec(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
