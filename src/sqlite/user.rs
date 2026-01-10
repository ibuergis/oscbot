use crate::{Error, sqlite};

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub osu_id: u32,
    pub discord_id: u64,
    pub is_blacklisted: bool
}

impl User {
    pub async fn create (
        osu_id: u32,
        discord_id: i64,
        is_blacklisted: bool,
    ) -> Result<User, Error> {
        let result = sqlx::query(r#"INSERT INTO user (osu_id, discord_id, is_blacklisted) VALUES (?, ?, ?)"#)
            .bind(osu_id)
            .bind(discord_id)
            .bind(is_blacklisted)
            .execute(sqlite::get_sqlite_instance()).await?;

        Ok(find(result.last_insert_rowid()).await?.expect("User must exist"))
    }

    pub async fn update(self) -> Result<(), Error> {
        let pool = sqlite::get_sqlite_instance();
    sqlx::query(
        r#"
        UPDATE user
        SET 
            osu_id  = ?,
            discord_id = ?,
            is_blacklisted = ?
        WHERE id = ?
            "#)
        .bind(self.osu_id)
        .bind(self.discord_id as i64)
        .bind(self.is_blacklisted)
        .bind(self.id)
        .execute(pool)
        .await?;

    Ok(())
    }
}


pub async fn find(id: i64) -> Result<Option<User>, sqlx::Error> {
    let pool = sqlite::get_sqlite_instance();
    let user = sqlx::query_as::<_, User>(r#"SELECT id, osu_id, discord_id, is_blacklisted FROM "user" WHERE id = ?"#)
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

pub async fn find_by_discord(discord_id: i64) -> Result<Option<User>, sqlx::Error> {
    let pool = sqlite::get_sqlite_instance();
    let user = sqlx::query_as::<_, User>(r#"SELECT id, osu_id, discord_id, is_blacklisted FROM "user" WHERE discord_id = ?"#)
        .bind(discord_id)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

pub async fn find_by_osu(osu_id: &u32) -> Result<Option<User>, sqlx::Error> {
    let pool = sqlite::get_sqlite_instance();
    let user = sqlx::query_as::<_, User>(r#"SELECT id, osu_id, discord_id, is_blacklisted FROM "user" WHERE osu_id = ?"#)
        .bind(osu_id)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

pub async fn find_by_blacklisted(is_blacklisted: bool) -> Result<Vec<User>, sqlx::Error> {
    let pool = sqlite::get_sqlite_instance();
    let user = sqlx::query_as::<_, User>(r#"SELECT id, osu_id, discord_id, is_blacklisted FROM "user" WHERE is_blacklisted = ?"#)
        .bind(is_blacklisted)
        .fetch_all(pool)
        .await?;

    Ok(user)
}