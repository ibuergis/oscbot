use poise::serenity_prelude::{self as serenity, CreateEmbed};
use rosu_v2::prelude as rosu;
use crate::{Data, Context, Error, defaults, embeds, osu};

async fn error_handler(error: poise::FrameworkError<'_, Data, Error>) {
    println!("Something went horribly wrong: {:?}", error);
}

#[poise::command(slash_command, rename = "suggest", subcommands("score"), required_permissions = "SEND_MESSAGES", on_error = "error_handler")]
pub async fn bundle(_ctx: Context<'_>, _arg: String) -> Result<(), Error> { Ok(()) }

/// Either submit score id or score file
#[poise::command(slash_command, ephemeral)]
pub async fn score(
    ctx: Context<'_>,
    #[description = "score id"] scoreid: Option<u64>,
    #[description = "score file"] _scorefile: Option<serenity::Attachment>,
    #[description = "reason"] _reason: Option<String>,
) -> Result<(), Error> {
    let score: rosu::Score;

    if scoreid.is_some() {
        let unwrapped_score_id = scoreid.unwrap();
        score = match osu::get_osu_instance().score(unwrapped_score_id).await {
            Ok(score) => score,
            Err(e) => {
                ctx.send(poise::CreateReply::default().embed(CreateEmbed::default().description(format!("Score with id {} does not exist", unwrapped_score_id)))).await?;
                return Err(Box::new(e));
            }
        };
    }
    else {
        ctx.say("not implemented yet").await?;
        return Ok(());
    }
    let embed = embeds::score_embed(&score).await?;
    defaults::SUGGESTIONS_CHANNEL.send_message(ctx, serenity::CreateMessage::new().embed(embed.footer(serenity::CreateEmbedFooter::new(format!("Requested by @{}", ctx.author().name))))).await?;
    ctx.send(poise::CreateReply::default().embed(CreateEmbed::default().description("Score has been submitted!"))).await?;
    Ok(())
}