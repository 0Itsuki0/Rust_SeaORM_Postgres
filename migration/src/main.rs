use anyhow::Result;
use dotenv::dotenv;
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let url = env::var("DATABASE_URL")?;

    let connect_options = ConnectOptions::new(url)
        .set_schema_search_path("public")
        .to_owned();

    let db = Database::connect(connect_options).await?;

    migration::Migrator::up(&db, None).await?;
    // migration::Migrator::down(&db, None).await?;

    Ok(())
}
