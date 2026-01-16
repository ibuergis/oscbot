use std::vec;

use poise::serenity_prelude::{self as serenity, CreateButton, CreateEmbed, ReactionType};
use rosu_v2::prelude as rosu;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use crate::{Context, Error, db::{self, entities::user}, defaults::{self, EMPTY_VALUE}, discord_helper::MessageState, embeds, generate::danser, osu};

#[poise::command(slash_command, rename = "suggest", subcommands("score"), required_permissions = "SEND_MESSAGES")]
pub async fn bundle(_ctx: Context<'_>, _arg: String) -> Result<(), Error> { Ok(()) }

/// Either submit score id or score file
#[poise::command(slash_command)]
pub async fn score(
    ctx: Context<'_>,
    #[description = "score id"] scoreid: Option<u64>,
    #[description = "score file"] scorefile: Option<serenity::Attachment>,
    #[description = "reason"] reason: Option<String>,
    #[description = "name of the skin. only accepts skins set by the player"] skin: Option<String>,
) -> Result<(), Error> {
    let embed: CreateEmbed;
    let mode: rosu::GameMode;
    let parameters: String;
    let requesting_user: u64 = ctx.author().id.into();
    ctx.defer().await?;
    
    if scoreid.is_some() {
        let unwrapped_score_id = scoreid.unwrap();
        if db::has_score(unwrapped_score_id.to_string()).await? {
            embeds::single_text_response(&ctx, &format!("Score {} has already been requested", unwrapped_score_id), MessageState::WARN, false).await;
            return Ok(());
        }
        let score: rosu::Score = match osu::get_osu_instance().score(unwrapped_score_id).await {
            Ok(score) => score,
            Err(_) => {
                embeds::single_text_response(&ctx, &format!("Score with id {} does not exist", unwrapped_score_id), MessageState::ERROR, false).await;
                return Ok(());
            }
        };

        if !score.has_replay {
            embeds::single_text_response(&ctx, "Score has no replay to download. Please provide the replay file", MessageState::ERROR, false).await;
            return Ok(());
        }

        let identifier = match skin {
            Some(identifier) => {
                match user::Entity::find().filter(user::Column::OsuId.eq(score.user_id as i64)).one(&db::get_db()).await? {
                    Some(user) => {
                        match db::get_skin_by_identifier(user, identifier.clone()).await? {
                            Some(_) => identifier,
                            None => {
                                embeds::single_text_response(&ctx, &format!("Skin with that name {} does not exist", identifier).to_string(), MessageState::ERROR, false).await;
                                return Ok(())
                            }
                        }
                    },
                    None => {
                        embeds::single_text_response(&ctx, &format!("Skin with that name {} does not exist", identifier).to_string(), MessageState::ERROR, false).await;
                        return Ok(())
                    }
                }
            },
            None => EMPTY_VALUE.into()
        };

        let map = osu::get_osu_instance().beatmap().map_id(score.map_id).await.expect("Beatmap exists");
        embed = embeds::score_embed_from_score(&score, &map, reason).await?;
        mode = score.mode;
        parameters = format!("{}:{}:{}:{}:{}", "scoreid".to_string(), score.id.to_string(), map.map_id, requesting_user, identifier);
        db::insert_score(unwrapped_score_id.to_string()).await?;

    }
    else if scorefile.is_some() {
        let bytes = scorefile.unwrap().download().await?;
        let replay = match osu_db::Replay::from_bytes(&bytes) {
            Ok(replay) => replay,
            Err(_) => {
                embeds::single_text_response(&ctx, "Replay could not be parsed", MessageState::ERROR, false).await;
                return Ok(());
            },
        };
        let default_checksum = "".to_string();
        let replay_checksum = replay.replay_hash.as_ref().unwrap_or(&default_checksum);
        if db::has_score(replay_checksum.clone()).await? {
            embeds::single_text_response(&ctx, "Score file has already been requested", MessageState::WARN, false).await;
            return Ok(());
        }
        let map: rosu::BeatmapExtended = match osu::get_beatmap_from_checksum(&replay.beatmap_hash).await {
            Some(map) => map,
            None => {
                embeds::single_text_response(&ctx, "Cannot find map related to the replay", MessageState::WARN, false).await;
                return Ok(());
            },
        };

        let identifier = match skin {
            Some(identifier) => {
                let player = match osu::get_osu_instance().user(replay.player_name.as_ref().unwrap()).await {
                    Ok(player) => player,
                    Err(_) => {
                        embeds::single_text_response(&ctx, &format!("Could not find player ``{}``", replay.player_name.unwrap()).to_string(), MessageState::ERROR, false).await;
                        return Ok(())
                    }
                };
                match user::Entity::find().filter(user::Column::OsuId.eq(player.user_id as i64)).one(&db::get_db()).await? {
                    Some(user) => {
                        match db::get_skin_by_identifier(user, identifier.clone()).await? {
                            Some(_) => identifier,
                            None => {
                                embeds::single_text_response(&ctx, &format!("Skin with that name {} does not exist", identifier).to_string(), MessageState::ERROR, false).await;
                                return Ok(())
                            }
                        }
                    },
                    None => {
                        embeds::single_text_response(&ctx, &format!("Skin with that name {} does not exist", identifier).to_string(), MessageState::ERROR, false).await;
                        return Ok(())
                    }
                }
            },
            None => EMPTY_VALUE.into()
        };
        danser::attach_replay(&map.checksum.as_ref().unwrap(), replay_checksum, &bytes).await.unwrap();
        embed = embeds::score_embed_from_replay_file(&replay, &map, reason).await?;
        mode = rosu::GameMode::from(replay.mode.raw());
        parameters = format!("{}:{}:{}:{}:{}", "replayfile".to_string(), replay_checksum.clone(), map.map_id, requesting_user, identifier);
        db::insert_score(replay_checksum.clone()).await?;
    }
    else {
        embeds::single_text_response(&ctx, "Please define scoreid or scorefile", MessageState::WARN, false).await;
        return Ok(());
    }

    let mut buttons: Vec<CreateButton> = vec![];
    
    if mode == rosu::GameMode::Osu {
        let approve_id = format!("approveWithUpload:{}", parameters);
        let approve_button = serenity::CreateButton::new(approve_id)
        .label("Approve with upload")
        .emoji(ReactionType::Unicode("✅".to_string()))
        .style(serenity::ButtonStyle::Success);
        buttons.push(approve_button);
    }
    else {
        let approve_id = format!("approveNoUpload:{}", parameters);
        let approve_button = serenity::CreateButton::new(approve_id)
        .label("Approve without upload")
        .emoji(ReactionType::Unicode("✅".to_string()))
        .style(serenity::ButtonStyle::Success);
        buttons.push(approve_button);
    }

    let decline_id = format!("decline:{}", parameters);
    let decline_button = serenity::CreateButton::new(decline_id)
        .label("Decline")
        .emoji(ReactionType::Unicode("💀".to_string()))
        .style(serenity::ButtonStyle::Danger);

    buttons.push(decline_button);

    let suggestion = serenity::CreateMessage::new()
            .embed(embed.footer(serenity::CreateEmbedFooter::new(format!("Requested by @{}", ctx.author().name))))
            .components(vec![serenity::CreateActionRow::Buttons(buttons)]);
    defaults::SUGGESTIONS_CHANNEL.send_message(ctx, suggestion).await?;
    embeds::single_text_response(&ctx, "Score has been requested!", MessageState::INFO, false).await;
    Ok(())
}