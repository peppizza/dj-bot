use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use songbird::input::Metadata;

use super::util::formatted_song_listing;
use crate::{checks::*, queue::get_queue_from_ctx_and_guild_id};

#[command]
#[checks(not_blacklisted)]
#[description = "Shows the currently queued tracks"]
#[bucket = "global"]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;
        let current_queue = queue.current_queue();

        if queue.is_empty() {
            msg.channel_id.say(ctx, "The queue is empty").await?;
            return Ok(());
        }

        let mut title_list = Vec::new();

        for handle in current_queue {
            let metadata = handle.name.clone();

            title_list.push(metadata);
        }

        let current = queue.current().unwrap();

        let metadata = current.metadata();

        let title = metadata.title.clone().unwrap();

        let mut response = formatted_song_listing(&title, &current, true, true, None).await?;

        title_list.remove(0);

        for (idx, title) in title_list.iter().enumerate() {
            response.push(format!("{} ", title));

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
