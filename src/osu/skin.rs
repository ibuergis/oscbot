use poise::ChoiceParameter;

use crate::{Error};

#[derive(Debug, Clone, Copy, PartialEq, ChoiceParameter, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum DEFAULTS {
    #[name = "Default"]
    DEFAULT,
    #[name = "NM"]
    NM,
    #[name = "HD"]
    HD,
    #[name = "DT"]
    DT,
    #[name = "HR"]
    HR,
    #[name = "EZ"]
    EZ,
    #[name = "HDDT"]
    HDDT,
    #[name = "HDHR"]
    HDHR,
    #[name = "No default"]
    NODEFAULT,
}

pub async fn download(url: &String) -> Result<Option<Vec<u8>>, Error> {
    let client = reqwest::Client::new();
    let resp = match client.get(url).send().await?.error_for_status() {
        Ok(response) => response,
        Err(_) => return Ok(None)
    };

    let bytes = match resp.bytes().await {
        Ok(bytes) => bytes.to_vec(),
        Err(_) =>  return Ok(None)
    };

    Ok(Some(bytes))
}