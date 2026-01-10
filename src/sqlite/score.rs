use crate::{Error, sqlite};

pub async fn insert_score(identifier: &String) -> Result<(), Error> {
    sqlx::query(r#"INSERT INTO score (identifier) VALUES (?)"#)
            .bind(identifier).execute(sqlite::get_sqlite_instance()).await?;

    Ok(())
}

pub async fn score_exists(identifier: &String) -> Result<bool, Error> {
    let count: i64 = sqlx::query_scalar(r#"SELECT COUNT(score.id) WHERE identifier = ?"#)
            .bind(identifier)
            .fetch_one(sqlite::get_sqlite_instance())
            .await?;
    Ok(count > 0)
}