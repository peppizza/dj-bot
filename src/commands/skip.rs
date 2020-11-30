use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, state::SongMetadataContainer};

#[command]
#[only_in(guilds)]
#[checks(Player)]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if let Some(handle) = queue.current() {
            {
                let data = ctx.data.read().await;
                let metadata_container_lock = data.get::<SongMetadataContainer>().unwrap().clone();
                let mut metadata_container = metadata_container_lock.write().await;

                metadata_container.remove(&handle.uuid());
            }
            queue.skip()?;
        } else {
            msg.reply_ping(ctx, "No song currently playing").await?;
            return Ok(());
        }

        msg.channel_id
            .say(ctx, format!("Song skipped: {} in queue.", queue.len() - 1))
            .await?;
    } else {
        msg.channel_id
            .say(ctx, "Not in a voice channel to skip")
            .await?;
    }

    Ok(())
}
