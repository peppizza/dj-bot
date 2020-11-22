use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::MessageBuilder,
};

use songbird::tracks::PlayMode;

use crate::state::SongMetadataContainer;

use super::util::format_duration_to_mm_ss;

#[command]
#[only_in(guilds)]
#[aliases("np", "playing")]
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

            let current_track_metadata = metadata_container.get(&current_track.uuid()).unwrap();

            let current_track_info = current_track.get_info()?.await?;

            let is_playing = matches!(current_track_info.playing, PlayMode::Play);

            let track_pos = current_track_info.position;

            let track_pos_mm_ss = format_duration_to_mm_ss(track_pos);

            let track_len = current_track_metadata.duration.unwrap();

            let track_len_mm_ss = format_duration_to_mm_ss(track_len);

            let track_title = current_track_metadata.title.clone();

            let mut response = MessageBuilder::new();

            if is_playing {
                response.push_bold(format!("[ {}/{} ]▶ ", track_pos_mm_ss, track_len_mm_ss));
            } else {
                response.push_bold(format!("[ {}/{} ]⏸ ", track_pos_mm_ss, track_len_mm_ss));
            }

            response.push(format!("{} ", track_title.unwrap()));

            let response = response.build();

            msg.channel_id.say(ctx, response).await?;
        } else {
            msg.channel_id.say(ctx, "No track playing").await?;
        }
    } else {
        msg.reply(ctx, "Nothing playing").await?;
    }

    Ok(())
}
