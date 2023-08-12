use sea_orm::DbConn;

use crate::persistence::image::image_saver::ImageSaver;

#[derive(Clone)]
pub struct PersistenceManager {
    pub(in crate::persistence) db_conn: DbConn,
}

impl PersistenceManager {
    pub fn new(db_conn: DbConn) -> Self {
        Self { db_conn }
    }

    pub fn make_image_saver(&self) -> ImageSaver {
        ImageSaver::new(self.db_conn.clone())
    }
}
