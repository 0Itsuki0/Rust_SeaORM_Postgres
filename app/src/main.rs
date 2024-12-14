use entity::{users, users::Entity as Users, users::Model};
use migration::{Migrator, MigratorTrait};

use anyhow::{Context, Result};
use dotenv::dotenv;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    DbBackend, EntityTrait, FromQueryResult, JsonValue, QueryFilter, QuerySelect, Statement,
};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let url = env::var("DATABASE_URL")?;
    let db = Database::connect(&url).await?;
    Migrator::up(&db, None).await?;

    let id = insert(&db).await?;
    let model = update(&db, &id).await?;
    select(&db).await?;
    select_name(&db).await?;
    delete(&db, &model).await?;
    Ok(())
}

async fn select_name(db: &DatabaseConnection) -> Result<()> {
    // sea orm
    let itsukis = Users::find()
        .filter(users::Column::Username.contains("itsuki"))
        .select_only()
        .column(users::Column::Username)
        .into_model::<JsonValue>()
        .all(db)
        .await?;
    println!("sea orm: all itsukis: {:?}", itsukis);

    // raw query
    let results: Vec<JsonValue> = JsonValue::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
                SELECT username
                FROM users
                WHERE username LIKE $1"#,
        ["%itsuki%".to_owned().into()],
    ))
    .all(db)
    .await?;

    println!("raw: all itsukis: {:?}", results);

    Ok(())
}

async fn select(db: &DatabaseConnection) -> Result<()> {
    // sea orm
    let itsukis: Vec<users::Model> = Users::find()
        .filter(users::Column::Username.contains("itsuki"))
        .all(db)
        .await?;
    println!("sea orm: all itsukis: {:?}", itsukis);

    let first_itsuki: Option<users::Model> = Users::find()
        .filter(users::Column::Username.contains("itsuki"))
        .one(db)
        .await?;
    println!("sea orm: first itsuki: {:?}", first_itsuki);

    // raw query
    let itsukis: Vec<users::Model> = Users::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
                SELECT *
                FROM users
                WHERE username LIKE $1"#,
            ["%itsuki%".to_owned().into()],
        ))
        .all(db)
        .await?;
    println!("raw: all itsukis: {:?}", itsukis);

    // way 2
    let statement = Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
                SELECT *
                FROM users
                WHERE username LIKE $1"#,
        ["%itsuki%".to_owned().into()],
    );
    let response = db
        .query_one(statement)
        .await?
        .context("failed to get query result")?;
    print_query_result(&response)?;

    Ok(())
}

async fn delete(db: &DatabaseConnection, model: &Model) -> Result<()> {
    let active_model = users::ActiveModel {
        id: ActiveValue::Set(model.id.to_owned()),
        username: ActiveValue::Set(model.username.to_owned()),
        ..Default::default()
    };
    let response = Users::delete(active_model.to_owned()).exec(db).await?;
    println!("{:?}", response);

    let response = Users::delete_by_id(model.id.to_owned()).exec(db).await?;
    println!("{:?}", response);

    let statement = Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
                DELETE FROM users
                WHERE username LIKE $1"#,
        ["%itsuki%".to_owned().into()],
    );
    let response = db.execute(statement).await?;
    println!("response: {:?}", response);

    Ok(())
}

async fn update(db: &DatabaseConnection, id: &str) -> Result<Model> {
    let updated_itsuki = users::ActiveModel {
        id: ActiveValue::Set(id.to_owned()),
        username: ActiveValue::Set("updated_itsuki_seaquery".to_owned()),
        ..Default::default()
    };

    let model = Users::update(updated_itsuki)
        .filter(users::Column::Username.contains("sea"))
        .exec(db)
        .await?;
    println!("{:?}", model);

    let statement = Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"UPDATE public.users
            SET username=$1
            WHERE id=$2
            RETURNING *;"#,
        [id.to_string().into(), "1".into()],
    );
    let response = db.execute(statement).await?;
    println!("{:?}", response);

    Ok(model)
}

async fn insert(db: &DatabaseConnection) -> Result<String> {
    let new_itsuki = users::ActiveModel {
        id: ActiveValue::Set(uuid::Uuid::new_v4().to_string()),
        username: ActiveValue::Set("new_itsuki_seaquery".to_owned()),
        ..Default::default()
    };
    let response = Users::insert(new_itsuki).exec(db).await?;
    println!("{:?}", response);
    let last_id = response.last_insert_id;

    let new_itsuki_json = users::ActiveModel::from_json(json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "username": "new_itsuki_json",
        "age": 1000000,
    }))?;
    let response = Users::insert(new_itsuki_json).exec(db).await?;
    println!("{:?}", response);

    let statement = Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"INSERT INTO users (id, username) VALUES ($1, $2) RETURNING *;"#,
        [
            uuid::Uuid::new_v4().to_string().into(),
            "new_itsuki_raw".into(),
        ],
    );
    // let response = db.execute(statement).await?;
    // println!("{:?}", response);
    let response = db
        .query_one(statement)
        .await?
        .context("failed to get query result!")?;
    print_query_result(&response)?;

    Ok(last_id)
}

fn print_query_result(result: &sea_orm::QueryResult) -> Result<()> {
    let columns = &result.column_names();
    let values = result.try_get_many::<(String, String, i64)>("", &result.column_names())?;
    println!("columns: {:#?}", columns);
    println!("values: {:#?}", values);
    Ok(())
}
