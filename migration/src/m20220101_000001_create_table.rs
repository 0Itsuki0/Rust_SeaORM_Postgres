use sea_orm::{DbBackend, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            "CREATE TABLE users (
                id VARCHAR PRIMARY KEY,
                username VARCHAR NOT NULL,
                age INT8 NOT NULL DEFAULT 0
            )",
        )
        .await?;

        // Construct a `Statement` if the SQL contains value bindings
        let statement = Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"INSERT INTO users (id, username) VALUES ($1, $2)"#,
            ["1".into(), "itsuki".into()],
        );
        db.execute(statement).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE users")
            .await?;
        Ok(())
    }
}
