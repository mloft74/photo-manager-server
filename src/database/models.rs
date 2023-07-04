use diesel::{pg::Pg, Insertable, Queryable, Selectable};

use crate::schema;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::images)]
#[diesel(check_for_backend(Pg))]
pub struct Image {
    pub id: i32,
    pub file_name: String,
}

#[derive(Insertable)]
#[diesel(table_name = schema::images)]
#[diesel(check_for_backend(Pg))]
pub struct InsertableImage {
    pub file_name: String,
}
