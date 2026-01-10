use futures_util::future::join_all;

use crate::{Error, osu::skin::DEFAULT, sqlite::{self, user::{self, User}}};

#[derive(Debug)]
pub struct Skin {
    pub id: i64,
    pub user: User,
    pub identifier: String,
    pub url: String,
    pub default: DEFAULT,
}

impl Skin {
    async fn from_db_skin (db_skin: &DbSkin) -> Skin {
        let user = user::find(db_skin.user).await.unwrap().unwrap();
        let default = match db_skin.default {
            Some(default) => default,
            None => DEFAULT::NODEFAULT
        };
        Skin {
            id: db_skin.id,
            user: user,
            identifier: db_skin.identifier.clone(),
            url: db_skin.url.clone(),
            default: default
        }
    }

    pub async fn create (
        user: &User,
        identifier: &String,
        url: &String,
        default: &DEFAULT
    ) -> Result<Skin, Error> {
        let skin = find_by_default(&user.id, &default).await?;
        if skin.is_some() {
            let mut unwrapped_skin = skin.unwrap();
            unwrapped_skin.default = DEFAULT::NODEFAULT;
            unwrapped_skin.update().await?;
        }

        let result = sqlx::query(r#"INSERT INTO skin (user, identifier, url, "default") VALUES (?, ?, ?, ?)"#)
            .bind(user.id)
            .bind(identifier)
            .bind(url)
            .bind(convert_default_for_db(*default))
            .execute(sqlite::get_sqlite_instance()).await?;

        Ok(find(&result.last_insert_rowid()).await?.expect("Skin must exist"))
    }

    pub async fn unsafe_update (self) -> Result<(), Error> {
        let pool = sqlite::get_sqlite_instance();
        sqlx::query(
            r#"
            UPDATE skin
            SET 
                identifier  = ?,
                url         = ?,
                "default"   = ?
            WHERE id = ?
                "#)
            .bind(self.identifier.clone())
            .bind(self.url.clone())
            .bind(convert_default_for_db(self.default))
            .bind(self.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update (self) -> Result<(), Error> {
        let skin = find_by_default(&self.user.id, &self.default).await?;
        if skin.is_some() {
            let mut unwrapped_skin = skin.unwrap();
            unwrapped_skin.default = DEFAULT::NODEFAULT;
            if self.id != unwrapped_skin.id {
                unwrapped_skin.unsafe_update().await?;
            }
        }
        self.unsafe_update().await
    }

    pub async fn delete (self) -> Result<(), Error> {
        let pool = sqlite::get_sqlite_instance();
        sqlx::query(
            r#"
            DELETE FROM skin
            WHERE id = ?
                "#)
            .bind(self.id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct DbSkin {
    id: i64,
    user: i64,
    identifier: String,
    url: String,
    default: Option<DEFAULT>,
}

fn convert_default_for_db(default: DEFAULT) -> Option<DEFAULT> {
    if default != DEFAULT::NODEFAULT {Some(default)} else {None}
}

pub async fn find(id: &i64) -> Result<Option<Skin>, sqlx::Error> {
    let pool = sqlite::get_sqlite_instance();
    let db_skin = sqlx::query_as::<_, DbSkin>(r#"SELECT id, user, identifier, url, "default" FROM "skin" WHERE id = ?"#)
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match db_skin {
        Some(db_skin) => Ok(Some(Skin::from_db_skin(&db_skin).await)),
        None => Ok(None)  
    }
}

pub async fn find_all_by_user(user_id: &i64) -> Result<Vec<Skin>, sqlx::Error> {
    let pool = sqlite::get_sqlite_instance();
    let db_skins = sqlx::query_as::<_, DbSkin>(r#"SELECT id, user, identifier, url, "default" FROM "skin" WHERE user = ?"#)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    let skins = db_skins.iter().map(async move |db_skin| Skin::from_db_skin(db_skin).await);
    Ok(join_all(skins).await)
}

pub async fn find_by_default(user_id: &i64, default: &DEFAULT) -> Result<Option<Skin>, sqlx::Error> {
    let pool = sqlite::get_sqlite_instance();
    let db_skin = sqlx::query_as::<_, DbSkin>(r#"SELECT id, user, identifier, url, "default" FROM "skin" WHERE user = ? AND "default" = ?"#)
        .bind(user_id)
        .bind(default)
        .fetch_optional(pool)
        .await?;

    match db_skin {
        Some(db_skin) => Ok(Some(Skin::from_db_skin(&db_skin).await)),
        None => Ok(None)  
    }
}

pub async fn find_by_identifier(user_id: &i64, identifier: &String) -> Result<Option<Skin>, sqlx::Error> {
    let pool = sqlite::get_sqlite_instance();
    let db_skin = sqlx::query_as::<_, DbSkin>(r#"SELECT id, user, identifier, url, "default" FROM "skin" WHERE user = ? AND lower(identifier) = lower(?)"#)
        .bind(user_id)
        .bind(identifier)
        .fetch_optional(pool)
        .await?;

    match db_skin {
        Some(db_skin) => Ok(Some(Skin::from_db_skin(&db_skin).await)),
        None => Ok(None)  
    }
}