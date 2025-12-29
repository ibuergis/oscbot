use poise::serenity_prelude as serenity;

use crate::{Context, Data, Error, embeds};
use crate::osu;

async fn error_handler(error: poise::FrameworkError<'_, Data, Error>) {
    println!("Something went horribly wrong: {:?}", error);
}

#[poise::command(slash_command, rename = "test", subcommands("osu_client"), on_error = "error_handler")]
pub async fn bundle(_ctx: Context<'_>, _arg: String) -> Result<(), Error> { Ok(()) }

#[poise::command(slash_command, on_error = "error_handler")]
pub async fn osu_client(ctx: Context<'_>) -> Result<(), Error> {
    let score = osu::get_osu_instance().score(1724681877).await.expect("Score should exist");
    let embed = embeds::score_embed(&score).await?;
    ctx.send(poise::CreateReply::default().embed(embed.footer(serenity::CreateEmbedFooter::new(format!("Requested by @{}", ctx.author().name))))).await?;
    Ok(())
}