use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, queue::QueueMap};

#[command]
#[checks(dj_only)]
#[description = "Stops the currently playing track, and clears the queue"]
#[aliases("leave", "die")]
#[bucket = "global"]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        {
            let data = ctx.data.read().await;
            let queue_container_lock = data.get::<QueueMap>().unwrap().clone();
            let mut queue_container = queue_container_lock.write().await;
            let queue = queue_container.remove(&guild_id).unwrap();
            queue.stop();

            let mut handler = handler_lock.lock().await;
            handler.remove_all_global_events();
        }

        manager.remove(guild_id).await?;

        msg.channel_id.say(ctx, "Cleared queue").await?;
    } else {
        msg.reply_ping(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}
