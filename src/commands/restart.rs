use std::time::Duration;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, queue::get_queue_from_ctx_and_guild_id};

#[command]
#[checks(dj_only)]
#[description = "Restarts the currently playing track"]
#[bucket = "global"]
async fn restart(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;

        let current = { queue.current().lock().clone() };

        if let Some(track_handle) = current {
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
