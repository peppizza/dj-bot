use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, queue::get_queue_from_ctx_and_guild_id};

use super::util::formatted_song_listing;

#[command]
#[aliases("np", "playing")]
#[checks(not_blacklisted)]
#[description = "Shows the currently playing track"]
#[bucket = "global"]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;
        if let Some(current_track) = queue.current() {
            let metadata = current_track.metadata();
            let title = metadata.title.clone().unwrap();

            let response = formatted_song_listing(&title, &current_track, true, false, None)
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
