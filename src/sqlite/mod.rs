use std::{fs::File, io::Write, path::Path, sync::OnceLock};

use sqlx::{Pool, Sqlite, sqlite::SqlitePool};

use crate::Error;

pub mod score;
pub mod user;
pub mod skin;

static SQLITE: OnceLock<Pool<Sqlite>> = OnceLock::new();

pub async fn initialize_sqlite() -> Result<(), Error> {
    if !Path::new("app.db").exists() {
        File::create("app.db").unwrap().flush()?;
    }
    let pool = SqlitePool::connect("sqlite://app.db").await?;

    // runs pending migrations from ./migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    match SQLITE.set(pool) {
        Ok(_) => return Ok(()),
        Err(_) => panic!("Sqlite could not be initialized"),
    };
}

pub fn get_sqlite_instance() -> &'static Pool<Sqlite> {
    SQLITE.get().expect("SQLITE is not initialized yet")
}