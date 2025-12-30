use poise::serenity_prelude as serenity;
use crate::{Context, Error};
use crate::defaults::REPLAY_ROLE;
use rosu_v2::prelude as rosu;
use crate::osu;
use crate::generate::thumbnail;

async fn has_replay_role(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => return Ok(false)
    };

    let member = guild_id.member(ctx, ctx.author().id).await?;
    if !member.roles.contains(&REPLAY_ROLE) {
        ctx.say("No permission L").await?;
        return Ok(false);
    }
    Ok(true)
}

#[poise::command(slash_command, rename = "replay", subcommands("generate"), check = "has_replay_role")]
pub async fn bundle(_ctx: Context<'_>, _arg: String) -> Result<(), Error> { Ok(()) }

#[poise::command(slash_command, subcommands("thumbnail"), check = "has_replay_role")]
pub async fn generate(_ctx: Context<'_>, _arg: String) -> Result<(), Error> { Ok(()) }

/// Either select score id or score file
#[poise::command(slash_command)]
pub async fn thumbnail(
    ctx: Context<'_>,
    #[description = "score id"] scoreid: Option<u64>,
    #[description = "score file"] scorefile: Option<serenity::Attachment>,
    #[description = "subtitle inside the thumbnail"] subtitle: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let score: rosu::Score;

    if scoreid.is_some() {
        let unwrapped_score_id = scoreid.unwrap();
        score = match osu::get_osu_instance().score(unwrapped_score_id).await {
            Ok(score) => score,
            Err(_) => {
                ctx.send(poise::CreateReply::default().embed(serenity::CreateEmbed::default().description(format!("Score with id {} does not exist", unwrapped_score_id)))).await?;
                return Ok(());
            }
        };
    }
    else if scorefile.is_some() {
        ctx.say("not implemented yet").await?;
        return Ok(());
    }
    else {

        return Ok(());
    }
    let map = osu::get_osu_instance().beatmap().map_id(score.map_id).await.expect("Beatmap exists");
    let image = thumbnail::generate_thumbnail_from_score(score, map, &subtitle.unwrap_or("".to_string())).await;
    ctx.send(poise::CreateReply::default().attachment(serenity::CreateAttachment::bytes(image, "thumbnail.png"))).await?;
    Ok(())
}