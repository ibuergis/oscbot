use poise::serenity_prelude::{self as serenity, CreateMessage};
use quick_xml::{events::Event, Reader};
use reqwest::StatusCode;
use std::{env, sync::{Mutex, OnceLock}};

use crate::{Error, defaults::NEW_VIDEOS_CHANNEL};

static LAST_VIDEO_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn value() -> &'static Mutex<Option<String>> {
    LAST_VIDEO_ID.get_or_init(|| Mutex::new(None))
}

fn set(s: String) {
    *value().lock().unwrap() = Some(s);
}

fn get_clone() -> Option<String> {
    value().lock().unwrap().clone()
}

pub async fn refresh_feed(ctx: &serenity::Context) -> Result<(), Error> {
    tracing::info!("checking youtube for new video uploads");
    let url = format!("https://www.youtube.com/feeds/videos.xml?channel_id={}", env::var("OSC_BOT_YOUTUBE_CHANNEL_ID").unwrap());
    let last_video_id = get_clone();
    tracing::info!(last = last_video_id);

    let c = reqwest::Client::new();
    let req = c.get(&url);

    let resp = req.send().await.unwrap();
    if resp.status() != StatusCode::OK { return Err(format!("http {}", resp.status()).into()); }

    let xml = resp.text().await?;
    let video_ids = get_video_ids(&xml)?;

    tracing::info!(video_ids = video_ids.join(", "));

    if video_ids.len() > 0 {
        set(video_ids[0].clone());
    }

    let unwrapped_video_id = match last_video_id {
        Some(video_id) => video_id,
        None => {
            tracing::info!("first loop... checking for video uploads is skipped");
            return Ok(())
        }
    };

    for video_id in video_ids {
        if video_id == unwrapped_video_id {
            tracing::info!("checking for new uploads has finished!");
            return Ok(())
        }
        tracing::info!(link = format!("https://youtu.be/{}", video_id), "New upload has been found!");
        NEW_VIDEOS_CHANNEL.send_message(ctx,
            CreateMessage::default().content(format!("A new score has been uploaded!\nhttps://youtu.be/{}", video_id))
        ).await?;
    }
    tracing::info!("checking for new uploads has finished!");
    Ok(())
}

fn get_video_ids(xml: &str) -> Result<Vec<String>, Error> {
    let mut r = Reader::from_str(xml);
    r.config_mut().trim_text(true);
    let mut b = Vec::new();
    let mut in_id = false;
    let mut video_ids: Vec<String> = vec![];
    loop {
        match r.read_event_into(&mut b) {
            Ok(Event::Start(e)) if e.name().as_ref().ends_with(b"videoId") => in_id = true,
            Ok(Event::End(e))   if e.name().as_ref().ends_with(b"videoId") => in_id = false,
            Ok(Event::Text(t))  if in_id => video_ids.push(t.decode()?.into_owned()),
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.into()),
            _ => {}
        }
        b.clear();
    }
    Ok(video_ids)
}
