use std::collections::HashMap;

use crate::firebase::get_firebase_instance;

pub async fn get_user_skin(osu_user_id: &String) -> Option<String> {
    get_firebase_instance().at("users").at(osu_user_id).at("skin").get::<Option<String>>().await.unwrap()
}

pub async fn save_skin(osu_user_id: &String, skin: &String) {
    get_firebase_instance().at("users").at(osu_user_id).set_with_key("skin", skin).await.unwrap();
}

pub async fn add_to_blacklist(discord_user_id: &String) {
    get_firebase_instance().at("blacklist").set_with_key(discord_user_id, &true).await.unwrap();
}

pub async fn remove_from_blacklist(discord_user_id: &String) {
    get_firebase_instance().at("blacklist").at(discord_user_id).delete().await.ok();
}

pub async fn get_blacklist() -> Option<HashMap<String, bool>> {
    get_firebase_instance().at("blacklist").get::<Option<HashMap<String, bool>>>().await.unwrap()
}

pub async fn user_is_in_blacklist(discord_user_id: &String) -> bool {
    match get_firebase_instance().at("blacklist").at(discord_user_id).get::<bool>().await.ok() {
        Some(_) => true,
        None => false,
    }
}