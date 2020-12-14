use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, data::SongAuthorContainer};

#[command]
#[checks(not_blacklisted)]
#[description = "Makes the bot leave the voice channel it is currently in"]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        {
            let handler = handler_lock.lock().await;
            let queue = handler.queue();

            if !queue.is_empty() {
                let current_queue = queue.current_queue();

                let data = ctx.data.read().await;
                let author_container_lock = data.get::<SongAuthorContainer>().unwrap().clone();
                let mut author_container = author_container_lock.write().await;

                for track in current_queue {
                    author_container.remove(&track.uuid());
                }
            }

            queue.stop();
        }

        manager.remove(guild_id).await?;

        msg.channel_id.say(ctx, "Left voice channel").await?;
    } else {
        msg.reply_ping(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}
