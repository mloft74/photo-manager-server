use diesel::{pg::Pg, Insertable, Queryable, Selectable};

use crate::schema;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::images)]
#[diesel(check_for_backend(Pg))]
pub struct Image {
    pub path: String,
}
