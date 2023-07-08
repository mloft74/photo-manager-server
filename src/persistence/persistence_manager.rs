use sea_orm::DatabaseConnection;

use crate::{domain::repos::RepoProvider, persistence::image_manager::ImageManager};

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

impl RepoProvider for PersistenceManager {
    type ImageRepoImpl = ImageManager;
    fn get_image_repo(&self) -> Self::ImageRepoImpl {
        self.new_image_manager()
    }
}
