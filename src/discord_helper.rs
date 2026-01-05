use poise::CreateReply;
use poise::serenity_prelude::{self as serenity, CacheHttp, CreateEmbed, CreateInteractionResponseMessage};

use crate::{Context, firebase};
use crate::{Data, Error, embeds::single_text_response};
use crate::defaults::{REPLAY_ROLE, SERVER};

pub struct ContextForFunctions<'a> {
    pub command_context: Option<Context<'a>>,
    pub reply: Option<poise::ReplyHandle<'a>>,
    pub event_context: Option<&'a (dyn serenity::CacheHttp + Send + Sync)>,
    pub component: Option<&'a serenity::ComponentInteraction>,
}

impl<'a> ContextForFunctions<'a> {
    pub async fn send(&mut self, embed: CreateEmbed) -> Result<(), Error> {
        match self.command_context {
            Some(ctx) => {
                self.reply = Some(ctx.send(CreateReply::default().embed(embed)).await.unwrap())
            }
            None => {
                self.component.unwrap().create_response(self.event_context.unwrap().http(), 
                serenity::CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::default().add_embed(embed)
                )
            ).await.unwrap()
            }
        }
        Ok(())
    }

    pub async fn edit(&self, embed: CreateEmbed) -> Result<(), Error> {
        match self.command_context {
            Some(ctx) => {
                self.reply.as_ref().unwrap().edit(ctx, CreateReply::default().embed(embed)).await.unwrap()
            }
            None => {
                self.component.unwrap().edit_response(self.event_context.unwrap().http(),
                serenity::EditInteractionResponse::default().embed(embed)
            ).await.unwrap();
            }
        }
        Ok(())
    }
}

#[derive(PartialEq)]
pub enum MessageState {
    SUCCESS,
    WARN,
    ERROR,
    INFO,
}

pub async fn handle_error(error: poise::FrameworkError<'_, Data, Error>) -> () {
    match error {
        poise::FrameworkError::CommandCheckFailed { .. } => return (),
        poise::FrameworkError::MissingBotPermissions { .. } => {
            match error.ctx() {
                Some(ctx) => {
                    single_text_response(&ctx, "Bot has missing permissions", MessageState::ERROR, true).await;
                },
            None => (),
            };
            return ()
        },
        poise::FrameworkError::MissingUserPermissions { .. } => return (),
        poise::FrameworkError::CommandPanic { payload, ctx, .. } => {
            println!("{:?}", payload);
            single_text_response(&ctx, "Something went wrong. blame Mikumin.", MessageState::ERROR, false).await;
        },
        _ => match error.ctx() {
                Some(ctx) => {
                    println!("{:?}", error);
                    single_text_response(&ctx, "Something went wrong. blame Mikumin.", MessageState::ERROR, false).await;
                },
                None => (),
        }
    };
}

pub async fn user_has_replay_role(ctx: impl CacheHttp, user: &serenity::User) -> Result<bool, Error> {
    let member = SERVER.member(ctx, user).await.unwrap();
    if !member.roles.contains(&REPLAY_ROLE) {
        return Ok(false);
    }
    Ok(true)
}

pub async fn global_check(ctx: Context<'_>) -> Result<bool, Error> {
    if firebase::user::user_is_in_blacklist(&ctx.author().id.to_string()).await {
        single_text_response(&ctx, "You are blacklisted", MessageState::INFO, true).await;
        return Ok(false)
    }

    Ok(true)
}