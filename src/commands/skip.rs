use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::checks::*;

#[command]
#[checks(dj_only)]
#[description = "Skips the currently playing track"]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if queue.current().is_some() {
            queue.skip()?;
        } else {
            msg.reply_ping(ctx, "No song currently playing").await?;
            return Ok(());
        }

        msg.channel_id
            .say(
                ctx,
                format!("Song skipped: {} songs left in queue.", queue.len() - 1),
            )
            .await?;
    } else {
        msg.channel_id
            .say(ctx, "Not in a voice channel to skip")
            .await?;
    }

    Ok(())
}
