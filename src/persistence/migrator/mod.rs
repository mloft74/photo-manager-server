// This is a mod.rs file to work with sea-orm-cli

use sea_orm_migration::prelude::*;

mod m20230707_014242_create_images_table;
mod m20230708_223818_add_image_size_columns;
mod m20230708_231248_remove_width_and_height_defaults;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230707_014242_create_images_table::Migration),
            Box::new(m20230708_223818_add_image_size_columns::Migration),
            Box::new(m20230708_231248_remove_width_and_height_defaults::Migration),
        ]
    }
}
