use rosu_v2::prelude as rosu;
use poise::serenity_prelude::{self as serenity, Colour};

use crate::osu;
use crate::{Context, Error};
use crate::discord_helper::MessageState;



static EMBED_COLOR: &[(&MessageState, Colour)] = &[
    (&MessageState::SUCCESS, Colour::new(0x873D48)),
    (&MessageState::WARN, Colour::new(0xFFA53F)),
    (&MessageState::ERROR, Colour::new(0xcc3300)),
    (&MessageState::INFO, Colour::new(0x1434A4)),
];

pub fn get_embed_color(message_state: &MessageState) -> Colour {
    EMBED_COLOR
        .iter()
        .find_map(|(k, v)| (*k == message_state).then_some(*v))
        .expect("State must have color")
}

pub async fn single_text_response(ctx: &Context<'_>, text: &str, message_state: MessageState ) {
    let _ = ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default().description(text).color(get_embed_color(&message_state))
        )
    ).await;
}

pub async fn score_embed(score: &rosu::Score) -> Result<serenity::CreateEmbed, Error> {
    let map = osu::get_osu_instance().beatmap().map_id(score.map_id).await.expect("Beatmap exists");
    let mapset = score.mapset.as_deref().expect("Mapset has not been found");
    let user = score.user.as_deref().expect("Mapset has not been found");
    let embed = serenity::CreateEmbed::default();
    let title = osu::formatter::map_title(&map);
    let author = serenity::CreateEmbedAuthor::new(format!("Score done by {}", user.username)).url(osu::formatter::score_url(&score.id));

    Ok(embed.author(author).color(get_embed_color(&MessageState::SUCCESS))
         .title(title)
         .url(map.url)
         .thumbnail(user.avatar_url.clone())
         .image(mapset.covers.card.clone())
         .field("Score:", score.score.to_string(), true)
         .field("Accuracy:", score.accuracy.to_string(), true)
         .field("Hits:", osu::formatter::osu_hits(&score.statistics), true)
         .field("Combo:", score.max_combo.to_string() + "x", true)
         .field("Mods:", osu::formatter::mods_string(&score.mods), true)
         .field("PP:", score.pp.unwrap_or(0.0).to_string(), true))
}