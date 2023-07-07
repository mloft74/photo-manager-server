use sea_orm::DatabaseConnection;

use crate::{domain::actions::ActionProvider, persistence::image_manager::ImageManager};

pub struct PersistenceManager {
    db_conn: DatabaseConnection,
}

impl PersistenceManager {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }
}

impl ActionProvider for PersistenceManager {
    type ImageSaverImpl = ImageManager;
    fn get_image_saver(&self) -> Self::ImageSaverImpl {
        ImageManager::new(self.db_conn.clone())
    }
}
