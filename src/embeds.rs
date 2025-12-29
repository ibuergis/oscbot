use rosu_v2::prelude as rosu;
use poise::serenity_prelude as serenity;

use crate::osu;
use crate::Error;

pub async fn score_embed(score: &rosu::Score) -> Result<serenity::CreateEmbed, Error> {
    let map = osu::get_osu_instance().beatmap().map_id(score.map_id).await.expect("Beatmap exists");
    let mapset = score.mapset.as_deref().expect("Mapset has not been found");
    let user = score.user.as_deref().expect("Mapset has not been found");
    let embed = serenity::CreateEmbed::default();
    let title = osu::formatter::map_title(&map);
    let author = serenity::CreateEmbedAuthor::new(format!("Score done by {}", user.username)).url(osu::formatter::score_url(&score.id));

    Ok(embed.author(author)
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