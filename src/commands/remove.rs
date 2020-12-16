use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::checks::*;

#[command]
#[checks(dj_only)]
#[description = "Removes a song from the queue, use ~queue to see what index to use"]
#[usage = "<index of song to remove>"]
#[bucket = "global"]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let index = args.single_quoted::<usize>()?;

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if !queue.is_empty() {
            if index == 0 {
                queue.skip()?;

                msg.channel_id.say(ctx, "Skipped the song").await?;
            } else if index > queue.len() {
                msg.reply_ping(ctx, format!("There is no song at index: {}", index))
                    .await?;
                return Ok(());
            } else {
                let track = queue.dequeue(index).unwrap();
                let metadata = track.metadata();
                let title = metadata.title.clone().unwrap_or_default();

                track.stop()?;

                msg.channel_id
                    .say(ctx, format!("Removed song: `{}`", title))
                    .await?;
            }
        } else {
            msg.channel_id.say(ctx, "The queue is empty").await?;
        }
    }

    Ok(())
}
