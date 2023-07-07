use sea_orm::DatabaseConnection;

use crate::{domain::actions::ActionProvider, persistence::image_manager::ImageManager};

pub struct PersistenceManager {
    db_conn: DatabaseConnection,
}

impl PersistenceManager {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }

    fn new_image_manager(&self) -> ImageManager {
        ImageManager::new(self.db_conn.clone())
    }
}

impl ActionProvider for PersistenceManager {
    type ImageGetterImpl = ImageManager;
    fn get_image_getter(&self) -> Self::ImageGetterImpl {
        self.new_image_manager()
    }

    type ImageSaverImpl = ImageManager;
    fn get_image_saver(&self) -> Self::ImageSaverImpl {
        self.new_image_manager()
    }
}
