use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use super::util::{format_duration_to_mm_ss, formatted_song_listing};
use crate::state::SongMetadataContainer;

#[command]
#[only_in(guilds)]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let current_queue = queue.current_queue();
        let data = ctx.data.read().await;
        let metadata_container_lock = data.get::<SongMetadataContainer>().unwrap().clone();
        let metadata_container = metadata_container_lock.read().await;

        if queue.is_empty() {
            msg.channel_id.say(ctx, "The queue is empty").await?;
            return Ok(());
        }

        let mut metadata_list = Vec::new();

        for handle in &current_queue {
            let metadata = metadata_container.get(&handle.uuid()).unwrap().clone();

            metadata_list.push(metadata);
        }

        let current = queue.current().unwrap();

        let metadata = metadata_container.get(&current.uuid()).unwrap();

        let mut response = formatted_song_listing(metadata, &current, true, true, None).await?;

        metadata_list.remove(0);

        for (idx, metadata) in metadata_list.iter().enumerate() {
            let track_length = metadata.duration.unwrap();
            let title = metadata.title.clone();

            let len_mm_ss = format_duration_to_mm_ss(track_length);

            response.push_bold(format!("[ {} ] ", len_mm_ss));

            response.push(format!("{} ", title.unwrap()));

            if idx != queue.len() {
                response.push_mono(format!("{}", idx + 1)).push("\n\n");
            } else {
                response.push_mono(format!("{}", idx + 1));
            }
        }

        let response = response.build();

        msg.channel_id.say(ctx, response).await?;
    } else {
        msg.reply(ctx, "Nothing playing").await?;
    }

    Ok(())
}
