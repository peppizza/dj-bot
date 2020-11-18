use std::time::Duration;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[only_in(guilds)]
async fn restart(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if let Some(track_handle) = queue.current() {
            track_handle.seek_time(Duration::from_secs(0))?;
        } else {
            msg.reply(ctx, "Nothing playing").await?;
        }
    } else {
        msg.reply(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}
