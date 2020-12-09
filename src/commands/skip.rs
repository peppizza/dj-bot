use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, state::SongAuthorContainer};

#[command]
#[checks(author_or_dj)]
#[description = "Skips the currently playing track"]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if let Some(handle) = queue.current() {
            {
                let data = ctx.data.read().await;
                let author_container_lock = data.get::<SongAuthorContainer>().unwrap().clone();
                let mut author_container = author_container_lock.write().await;

                author_container.remove(&handle.uuid());
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
