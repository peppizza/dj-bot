use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, queue::get_queue_from_ctx_and_guild_id};

#[command]
#[checks(dj_only)]
#[description = "Skips the currently playing track"]
#[bucket = "global"]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;

        if queue.current().await.is_some() {
            queue.skip().await?;
        } else {
            msg.reply_ping(ctx, "No song currently playing").await?;
            return Ok(());
        }

        msg.channel_id
            .say(
                ctx,
                format!(
                    "Song skipped: {} songs left in queue.",
                    queue.len().await - 1
                ),
            )
            .await?;
    } else {
        msg.channel_id
            .say(ctx, "Not in a voice channel to skip")
            .await?;
    }

    Ok(())
}
