use std::time::Duration;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::checks::*;

#[command]
#[checks(Player)]
#[description = "Restarts the currently playing track"]
#[bucket = "player"]
async fn restart(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if let Some(track_handle) = queue.current() {
            msg.channel_id.say(ctx, "Restarting track...").await?;
            track_handle.seek_time(Duration::from_secs(0))?;
        } else {
            msg.reply_ping(ctx, "Nothing playing").await?;
        }
    } else {
        msg.reply_ping(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}
