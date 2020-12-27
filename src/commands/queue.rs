use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use songbird::input::Metadata;

use super::util::{format_duration_to_mm_ss, formatted_song_listing};
use crate::checks::*;

#[command]
#[checks(not_blacklisted)]
#[description = "Shows the currently queued tracks"]
#[bucket = "global"]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let current_queue = queue.current_queue();

        if queue.is_empty() {
            msg.channel_id.say(ctx, "The queue is empty").await?;
            return Ok(());
        }

        let mut metadata_list = current_queue
            .iter()
            .map(|handle| handle.metadata().clone())
            .collect::<Vec<Metadata>>();

        let current = queue.current().unwrap();

        let metadata = current.metadata();

        let mut response = formatted_song_listing(metadata, &current, true, true, None).await?;

        metadata_list.remove(0);

        for (idx, metadata) in metadata_list.iter().enumerate() {
            let track_length = metadata.duration.unwrap_or_default();
            let title = metadata.title.clone();

            let len_mm_ss = format_duration_to_mm_ss(track_length);

            response.push_bold(format!("[ {} ] ", len_mm_ss));

            response.push(format!("{} ", title.unwrap_or_default()));

            if idx != queue.len() {
                response.push_mono(format!("{}", idx + 1)).push("\n\n");
            } else {
                response.push_mono(format!("{}", idx + 1));
            }
        }

        let response = response.build();

        msg.channel_id.say(ctx, response).await?;
    } else {
        msg.reply_ping(ctx, "Nothing playing").await?;
    }

    Ok(())
}
