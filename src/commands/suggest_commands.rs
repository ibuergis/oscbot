use std::vec;

use poise::serenity_prelude as serenity;
use rosu_v2::prelude as rosu;
use crate::{Context, Error, defaults, discord_helper::MessageState, embeds, firebase, osu};

#[poise::command(slash_command, rename = "suggest", subcommands("score"), required_permissions = "SEND_MESSAGES")]
pub async fn bundle(_ctx: Context<'_>, _arg: String) -> Result<(), Error> { Ok(()) }

/// Either submit score id or score file
#[poise::command(slash_command)]
pub async fn score(
    ctx: Context<'_>,
    #[description = "score id"] scoreid: Option<u64>,
    #[description = "score file"] _scorefile: Option<serenity::Attachment>,
    #[description = "reason"] _reason: Option<String>,
) -> Result<(), Error> {
    let score: rosu::Score;

    if scoreid.is_some() {
        let unwrapped_score_id = scoreid.unwrap();
        if firebase::scores::score_already_saved(unwrapped_score_id).await {
            embeds::single_text_response(&ctx, &format!("Score {} has already been requested", unwrapped_score_id), MessageState::WARN).await;
            return Ok(());
        }
        score = match osu::get_osu_instance().score(unwrapped_score_id).await {
            Ok(score) => score,
            Err(_) => {
                embeds::single_text_response(&ctx, &format!("Score with id {} does not exist", unwrapped_score_id), MessageState::ERROR).await;
                return Ok(());
            }
        };
    }
    else {
        ctx.say("not implemented yet").await?;
        return Ok(());
    }
    let button_id = format!("thumbnail:{}", score.id);
    let button = serenity::CreateButton::new(button_id)
            .label("Render Thumbnail")
            .emoji(crate::emojis::SATA_ANDAGI)
            .style(serenity::ButtonStyle::Primary);

    let embed = embeds::score_embed(&score).await?;
    firebase::scores::insert_score(&score.id).await;
    defaults::SUGGESTIONS_CHANNEL.send_message(ctx, serenity::CreateMessage::new()
        .embed(embed.footer(serenity::CreateEmbedFooter::new(format!("Requested by @{}", ctx.author().name))))
        .components(vec![serenity::CreateActionRow::Buttons(vec![button])])
    ).await?;
    embeds::single_text_response(&ctx, "Score has been requested!", MessageState::INFO).await;
    Ok(())
}