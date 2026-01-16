use poise::{CreateReply, serenity_prelude::{self as serenity, CreateEmbed, Mentionable}};

use crate::{Context, Error, db::{self, entities::user}, discord_helper::{MessageState, user_has_replay_role}, embeds::single_text_response, osu};

use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

async fn has_replay_role(ctx: Context<'_>) -> Result<bool, Error> {
    if !user_has_replay_role(ctx, ctx.author()).await.unwrap() {
        single_text_response(&ctx, "No permission L", MessageState::INFO, true).await;
        return Ok(false);
    }
    Ok(true)
}


#[poise::command(slash_command, rename = "admin", subcommands("blacklist"), check="has_replay_role")]
pub async fn bundle(_ctx: Context<'_>, _arg: String) -> Result<(), Error> { Ok(()) }

#[poise::command(slash_command, subcommands("add", "remove", "list"))]
pub async fn blacklist(_ctx: Context<'_>, _arg: String) -> Result<(), Error> { Ok(()) }

#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    user: serenity::Member
) -> Result<(), Error> {
    let user_id: i64 = user.user.id.into();

    let mut db_user: user::ActiveModel = match db::get_user_by_discord_id(user_id).await? {
        Some(user) => {
            user.into()
        },
        None => {
            let player = match osu::get_osu_instance().user(user.display_name()).await {
                Ok(player) => player.user_id,
                _ => {
                    single_text_response(&ctx, &format!("User {} has the wrong username. please inform a moderator!", user.mention().to_string()), MessageState::SUCCESS, false).await;
                    0
                }
            };
            
            user::ActiveModel {
                discord_id: Set(user_id),
                osu_id: Set(player as i64),
                ..Default::default()
            }.insert(&db::get_db()).await?.into()
        }
    };

    db_user.is_blacklisted = Set(true);
    db_user.update(&db::get_db()).await?;

    single_text_response(&ctx, &format!("User {} has been blacklisted", user.mention().to_string()), MessageState::SUCCESS, false).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    user: serenity::Member
) -> Result<(), Error> {
    let user_id: i64 = user.user.id.into();
    let mut db_user: user::ActiveModel = match db::get_user_by_discord_id(user_id).await? {
        Some(user) => {
            user.into()
        },
        None => {
            let player = match osu::get_osu_instance().user(user.display_name()).await {
                Ok(player) => player.user_id,
                _ => {
                    single_text_response(&ctx, &format!("User {} has the wrong username. please inform a moderator!", user.mention().to_string()), MessageState::SUCCESS, false).await;
                    0
                }
            };
            
            user::ActiveModel {
                discord_id: Set(user_id),
                osu_id: Set(player as i64),
                ..Default::default()
            }.insert(&db::get_db()).await?.into()
        }
    };

    db_user.is_blacklisted = Set(false);
    db_user.update(&db::get_db()).await?;
    single_text_response(&ctx, &format!("User {} has been removed from the blacklist", user.mention().to_string()), MessageState::SUCCESS, false).await;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let blacklist = user::Entity::find().filter(user::Column::IsBlacklisted.eq(true)).all(&db::get_db()).await?;

    if blacklist.is_empty() {
        single_text_response(&ctx, "The blacklist is empty", MessageState::INFO, false).await;
        return Ok(());
    }

    let mut blacklist_content = "".to_string();
    for user in blacklist {
        blacklist_content = format!("{}<@{}>\n", blacklist_content, user.discord_id);
    }

    let blacklist_embed = CreateEmbed::default().title("Blacklist").description(blacklist_content);
    ctx.send(CreateReply::default().embed(blacklist_embed)).await?;
    Ok(())
}