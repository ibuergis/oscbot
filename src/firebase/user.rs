use std::collections::HashMap;

use firebase_rs::Firebase;
use poise::ChoiceParameter;

use crate::{firebase::get_firebase_instance, osu::skin::DEFAULTS};

fn path_to_skins(osu_user_id: &String) -> Firebase {
    get_firebase_instance()
        .at("users")
        .at(osu_user_id)
        .at("skins")
}

pub async fn get_skin_by_identifier(osu_user_id: &String, identifier: &String) -> Option<String> {
    match path_to_skins(osu_user_id)
        .at("list")
        .at(identifier)
        .get::<String>().await {
        Ok(skin) => Some(skin),
        Err(_) => None
    }
}

pub async fn get_skin_by_default(osu_user_id: &String, default: &DEFAULTS) -> Option<String> {
    match path_to_skins(osu_user_id)
        .at("defaults")
        .at(default.name())
        .get::<String>().await {
        Ok(skin) => Some(skin),
        Err(_) => None
    }
}

pub async fn save_skin(osu_user_id: &String, skin: &String, identifier: &String, default: &DEFAULTS) {
    path_to_skins(osu_user_id)
        .at("list")
        .at(identifier)
        .set_with_key("url", skin)
        .await
        .unwrap();

    path_to_skins(osu_user_id)
        .at("list")
        .at(identifier)
        .set_with_key("default", &default.name().to_string());
    
    if *default != DEFAULTS::NODEFAULT {
        path_to_skins(osu_user_id)
        .at("defaults")
        .set_with_key(default.name(), skin).await.unwrap();
    }
}

pub async fn delete_skin(osu_user_id: &String, skin: &String, identifier: &String, default: &DEFAULTS) {

}

pub async fn add_to_blacklist(discord_user_id: &String) {
    get_firebase_instance().at("blacklist").set_with_key(discord_user_id, &true).await.unwrap();
}

pub async fn remove_from_blacklist(discord_user_id: &String) {
    get_firebase_instance().at("blacklist").at(discord_user_id).delete().await.ok();
}

pub async fn get_blacklist() -> Option<HashMap<String, bool>> {
    match get_firebase_instance().at("blacklist").get::<HashMap<String, bool>>().await {
        Ok(blacklist) => Some(blacklist),
        Err(_) => None
    }
}

pub async fn user_is_in_blacklist(discord_user_id: &String) -> bool {
    match get_firebase_instance().at("blacklist").at(discord_user_id).get::<bool>().await.ok() {
        Some(_) => true,
        None => false,
    }
}