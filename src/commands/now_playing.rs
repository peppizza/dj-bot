use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, state::SongMetadataContainer};

use super::util::formatted_song_listing;

#[command]
#[only_in(guilds)]
#[aliases("np", "playing")]
#[checks(Player)]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if let Some(current_track) = queue.current() {
            let data = ctx.data.read().await;
            let metadata_container_lock = data.get::<SongMetadataContainer>().unwrap().clone();
            let metadata_container = metadata_container_lock.read().await;

            let current_track_metadata = metadata_container
                .get(&current_track.uuid())
                .unwrap()
                .metadata
                .clone();

            let response =
                formatted_song_listing(&current_track_metadata, &current_track, true, false, None)
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
