use std::sync::LazyLock;
use poise::serenity_prelude as serenity;


pub static REPLAY_ROLE: LazyLock<serenity::RoleId> = LazyLock::new(|| {
    let id: u64 = std::env::var("REPLAY_ADMIN_ROLE")
        .expect("REPLAY_ADMIN_ROLE not set")
        .parse()
        .expect("REPLAY_ADMIN_ROLE must be u64");
    serenity::RoleId::new(id)
});

pub static SUGGESTIONS_CHANNEL: LazyLock<serenity::ChannelId> = LazyLock::new(|| {
    let id: u64 = std::env::var("OSC_BOT_REQUEST_CHANNEL")
        .expect("OSC_BOT_REQUEST_CHANNEL not set")
        .parse()
        .expect("OSC_BOT_REQUEST_CHANNEL must be u64");
    serenity::ChannelId::new(id)
});
