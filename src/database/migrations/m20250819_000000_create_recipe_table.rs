use sea_orm::DbErr;
use sea_orm_migration::{MigrationName, MigrationTrait, SchemaManager};

pub struct Migration;

#[async_trait::async_trait]
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250819_000000_create_recipe_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}