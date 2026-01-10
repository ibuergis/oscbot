use poise::{CreateReply, serenity_prelude::{self as serenity, CreateEmbed, CreateEmbedAuthor}};
use url::Url;

use crate::{Context, Error, discord_helper::MessageState, embeds::single_text_response, osu::{self, skin::DEFAULT}, sqlite};
use crate::discord_helper::user_has_replay_role;

async fn has_replay_role(ctx: Context<'_>) -> Result<bool, Error> {
    if !user_has_replay_role(ctx, ctx.author()).await.unwrap() {
        single_text_response(&ctx, "No permission L", MessageState::INFO, true).await;
        return Ok(false);
    }
    Ok(true)
}

fn is_url(s: &str) -> bool {
    Url::parse(s).is_ok()
}


#[poise::command(slash_command, rename = "skin", subcommands("set", "get", "remove"), required_permissions = "SEND_MESSAGES")]
pub async fn bundle(_ctx: Context<'_>, _arg: String) -> Result<(), Error> { Ok(()) }

/// Save a url to your skin
#[poise::command(slash_command)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "link to your skin"] url: String,
    #[description = "Name to reference your skin"] identifier: String,
    #[description = "default for when you upload the gamemode. If HDDT is not set, the DT skin will be used instead."] default: osu::skin::DEFAULT,
    #[description = "Desired member (uploaders only)"] member: Option<serenity::Member>,
) -> Result<(), Error> {
    ctx.defer().await?;

    if member.is_some() {
        if !user_has_replay_role(ctx.http(), ctx.author()).await? {
            single_text_response(&ctx, "No permission L", MessageState::INFO, true).await;
            return Ok(())
        }
    }

    if !is_url(&url) || !url.starts_with("https://git.sulej.net/") || !url.ends_with(".osk") {
        single_text_response(&ctx, "Please enter the download link to your skin in https://git.sulej.net/", MessageState::WARN, false).await;
        return Ok(());
    }

    let username = match &member {
        Some(member) => member.display_name().to_string(),
        None => {
            let member = ctx.author_member().await.unwrap();
            member.display_name().to_string()
        }
    };

    let user_id = match &member {
        Some(member) => member.user.id,
        None => ctx.author().id
    };

    let user = match osu::get_osu_instance().user(username).await {
        Ok(user) => user,
        Err(_) =>  {
            single_text_response(&ctx, "Your username is not related to your osu!account. Please inform the mods to rename you!", MessageState::WARN, false).await;
            return Ok(())
        }
    };

    let user = match sqlite::user::find_by_discord(user_id.into()).await.unwrap() {
        Some(user) => user,
        None => sqlite::user::User::create(user.user_id, user_id.into(), false).await.unwrap()
    };

    let skin_upload_successful = osu::skin::download(&url).await?.is_some();

    if !skin_upload_successful {
        single_text_response(&ctx, "The skin file could not be downloaded", MessageState::WARN, false).await;
        return Ok(());
    }

    match sqlite::skin::find_by_identifier(&user_id.into(), &identifier).await? {
        Some(mut skin) => {
            skin.url = url;
            skin.default = default;
            skin.update().await?;
        },
        None => {
            sqlite::skin::Skin::create(&user, &identifier, &url, &default).await.unwrap();
        },
    };

    single_text_response(&ctx, "Skin has been saved", MessageState::SUCCESS, false).await;
    Ok(())
}

/// Get the url to a members skins
#[poise::command(slash_command, check = "has_replay_role")]
pub async fn get(
    ctx: Context<'_>,
    #[description = "Desired member"] member: Option<serenity::Member>,
    #[description = "leave empty for all skins"] identifier: Option<String>,
) -> Result<(), Error> {
    let username = match &member {
        Some(member) => member.display_name().to_string(),
        None => {
            let member = ctx.author_member().await.unwrap();
            member.display_name().to_string()
        }
    };

    let user_id = match &member {
        Some(member) => member.user.id,
        None => ctx.author().id
    };

    let player = match osu::get_osu_instance().user(&username).await {
        Ok(user) => user,
        Err(_) =>  {
            single_text_response(&ctx, "Your username is not related to your osu!account. Please inform the mods to rename you!", MessageState::SUCCESS, false).await;
            return Ok(())
        }
    };

    let user = match sqlite::user::find_by_discord(user_id.into()).await.unwrap() {
        Some(user) => user,
        None => sqlite::user::User::create(player.user_id, user_id.into(), false).await.unwrap()
    };
    
    let skins = match identifier {
        Some(identifier) => {
            match sqlite::skin::find_by_identifier(&user.id, &identifier).await? {
                Some(skin) => vec![skin],
                None => {
                    single_text_response(&ctx, "This user has not a skin with that identifier", MessageState::INFO, false).await;
                    return Ok(())
                }
            }
        },
        None => {
            sqlite::skin::find_all_by_user(&user.id).await?
        }
    };

    if skins.is_empty() {
        single_text_response(&ctx, "This user has not saved a skin", MessageState::INFO, false).await;
        return Ok(());
    }

    let mut embed = CreateEmbed::default().author(CreateEmbedAuthor::new(format!("Skins: {}", username)));
    for skin in skins {
        let default_text: String = if skin.default != DEFAULT::NODEFAULT {format!("({})", skin.default.to_string())} else {"".to_string()};
        embed = embed.field("", format!("[{} {}]({})", skin.identifier, default_text, skin.url), false);
    }

    ctx.send(CreateReply::default().embed(embed)).await.unwrap();
    Ok(())
}

/// Get the url to a members skins
#[poise::command(slash_command, check = "has_replay_role")]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "leave empty for all skins"] identifier: String,
    #[description = "Desired member (uploaders only)"] member: Option<serenity::Member>,
) -> Result<(), Error> {
    if member.is_some() {
        if !user_has_replay_role(ctx.http(), ctx.author()).await? {
            single_text_response(&ctx, "No permission L", MessageState::INFO, true).await;
            return Ok(())
        }
    }

    let username = match &member {
        Some(member) => member.display_name().to_string(),
        None => {
            let member = ctx.author_member().await.unwrap();
            member.display_name().to_string()
        }
    };

    let user_id = match &member {
        Some(member) => member.user.id,
        None => ctx.author().id
    };

    let player = match osu::get_osu_instance().user(&username).await {
        Ok(user) => user,
        Err(_) =>  {
            single_text_response(&ctx, "Your username is not related to your osu!account. Please inform the mods to rename you!", MessageState::SUCCESS, false).await;
            return Ok(())
        }
    };

    let user = match sqlite::user::find_by_discord(user_id.into()).await.unwrap() {
        Some(user) => user,
        None => sqlite::user::User::create(player.user_id, user_id.into(), false).await.unwrap()
    };
    
    let skin =  sqlite::skin::find_by_identifier(&user.id, &identifier).await?;

    match skin {
        Some(skin) => {
            skin.delete().await?;
            single_text_response(&ctx, &format!("Skin ```{}``` has been removed!", identifier), MessageState::SUCCESS, false).await;
        },
        None => {
            single_text_response(&ctx, &format!("Skin ```{}``` does not exist!", identifier), MessageState::INFO, false).await
        }
    }
    Ok(())
}
