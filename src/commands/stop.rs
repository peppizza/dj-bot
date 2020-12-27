use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, data::StopContainer, queue::QueueMap};

#[command]
#[checks(dj_only)]
#[description = "Stops the currently playing track, and clears the queue"]
#[aliases("leave", "die")]
#[bucket = "global"]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        {
            let data = ctx.data.read().await;
            let channel_container_lock = data.get::<StopContainer>().unwrap().clone();
            let mut channel_container = channel_container_lock.lock().await;
            let queue_container_lock = data.get::<QueueMap>().unwrap().clone();
            let mut queue_container = queue_container_lock.write().await;
            let queue = queue_container.remove(&guild_id).unwrap();
            queue.stop().await;

            let channel = channel_container.remove(&guild_id).unwrap();

            channel.send_async(()).await.unwrap();
        }

        manager.remove(guild_id).await?;

        msg.channel_id.say(ctx, "Cleared queue").await?;
    } else {
        msg.reply_ping(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}
