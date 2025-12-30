use crate::firebase::get_firebase_instance;

pub async fn score_already_saved(score_id: u64) -> bool {
    match get_firebase_instance().at("checked_scores").at(&score_id.to_string()).get::<Option<bool>>().await {
        Ok(Some(true)) => true,
        _ => false,
    }
}

pub async fn insert_score(score_id: &u64) {
    get_firebase_instance().at("checked_scores").set_with_key(&score_id.to_string(), &true).await.unwrap();
}