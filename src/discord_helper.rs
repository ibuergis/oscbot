use crate::{Data, Error, embeds::single_text_response};

#[derive(PartialEq)]
pub enum MessageState {
    SUCCESS,
    WARN,
    ERROR,
    INFO,
}

pub async fn handle_error(error: poise::FrameworkError<'_, Data, Error>) -> () {
    match error.ctx() {
        Some(ctx) => single_text_response(&ctx, "Something went wrong. blame Mikumin.", MessageState::ERROR).await,
        None => return ()
    };
}