use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::checks::*;

use super::util::formatted_song_listing;

#[command]
#[only_in(guilds)]
#[aliases("np", "playing")]
#[checks(Player)]
#[description = "Shows the currently playing track"]
#[usage = "| np | playing"]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if let Some(current_track) = queue.current() {
            let metadata = current_track.metadata();

            let response = formatted_song_listing(metadata, &current_track, true, false, None)
                .await?
                .build();

            msg.channel_id.say(ctx, response).await?;
        } else {
            msg.channel_id.say(ctx, "No track playing").await?;
        }
    } else {
        msg.reply(ctx, "Nothing playing").await?;
    }

    Ok(())
}
