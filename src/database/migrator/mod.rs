// This is a mod.rs file to work with sea-orm-cli

use sea_orm_migration::prelude::*;

mod m20230707_014242_create_images_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20230707_014242_create_images_table::Migration)]
    }
}