use sea_orm::DbConn;

#[derive(Clone)]
pub struct PersistenceManager {
    pub(in crate::persistence) db_conn: DbConn,
}

impl PersistenceManager {
    pub fn new(db_conn: DbConn) -> Self {
        Self { db_conn }
    }
}
