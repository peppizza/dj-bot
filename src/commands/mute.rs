use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::checks::*;

#[command]
#[only_in(guilds)]
#[checks(Player)]
#[description = "Mutes the bot"]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            msg.reply_ping(ctx, "Not in a voice channel").await?;

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        handler.mute(false).await?;
        msg.channel_id.say(ctx, "Unmuted").await?;
    } else {
        handler.mute(true).await?;
        msg.channel_id.say(ctx, "Muted").await?;
    }

    Ok(())
}
