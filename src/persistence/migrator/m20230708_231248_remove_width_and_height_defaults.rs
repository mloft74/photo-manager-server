use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("ALTER TABLE \"images\" ALTER COLUMN \"width\" DROP DEFAULT, ALTER COLUMN \"height\" DROP DEFAULT;").await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("ALTER TABLE \"images\" ALTER COLUMN \"width\" SET DEFAULT 0, ALTER COLUMN \"height\" SET DEFAULT 0;").await?;

        Ok(())
    }
}
