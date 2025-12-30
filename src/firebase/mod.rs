use std::sync::OnceLock;

use firebase_rs::Firebase;

use crate::{Error};

pub mod scores;

static FIREBASE: OnceLock<Firebase> = OnceLock::new();

pub async fn initialize_firebase() -> Result<(), Error> {
    let project_url = std::env::var("OSC_BOT_FIREBASE_PROJECT_URL").expect("Firebase project must be defined");
    let db_secret = std::env::var("OSC_BOT_FIREBASE_AUTH_KEY").expect("Firebase Auth key must be defined");

    match FIREBASE.set(Firebase::auth(&project_url, &db_secret).unwrap()) {
        Ok(_) => return Ok(()),
        Err(_) => panic!("Firebase could not be initialized"),
    };
}

pub fn get_firebase_instance() -> &'static Firebase {
    FIREBASE.get().expect("FIREBASE is not initialized yet")
}