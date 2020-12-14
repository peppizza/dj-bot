use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, data::SongAuthorContainer, util::remove_entries_from_author_container};

#[command]
#[checks(not_blacklisted)]
#[description = "Stops the currently playing track, and clears the queue"]
#[aliases("leave", "die")]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let data = ctx.data.read().await;
        let author_container_lock = data.get::<SongAuthorContainer>().unwrap().clone();
        remove_entries_from_author_container(handler_lock, author_container_lock).await;

        manager.remove(guild_id).await?;

        msg.channel_id.say(ctx, "Cleared queue").await?;
    } else {
        msg.reply_ping(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}
