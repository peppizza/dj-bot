use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, state::SongMetadataContainer};

#[command]
#[only_in(guilds)]
#[checks(Player)]
#[description = "Stops the currently playing track, and clears the queue"]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        {
            let handler = handler_lock.lock().await;
            let queue = handler.queue();
            let current_queue = queue.current_queue();

            if !current_queue.is_empty() {
                let data = ctx.data.read().await;
                let metadata_container_lock = data.get::<SongMetadataContainer>().unwrap().clone();
                let mut metadata_container = metadata_container_lock.write().await;

                for track in current_queue {
                    metadata_container.remove(&track.uuid());
                }
            }

            queue.stop();
        }

        manager.remove(guild_id).await?;

        msg.channel_id.say(ctx, "Cleared queue").await?;
    } else {
        msg.channel_id
            .say(ctx, "Not in a voice channel to play in")
            .await?;
    }

    Ok(())
}
