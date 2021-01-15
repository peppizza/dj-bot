use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use songbird::tracks::PlayMode;

use crate::{checks::*, queue::get_queue_from_ctx_and_guild_id};

#[command]
#[checks(dj_only)]
#[description = "Pauses/Resumes the currently playing track"]
#[bucket = "global"]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;

        let current = { queue.current().lock().clone() };

        if let Some(handle) = current {
            match handle.get_info().await?.playing {
                PlayMode::Play => {
                    handle.pause()?;
                    msg.channel_id.say(ctx, "Paused").await?;
                }
                PlayMode::Pause => {
                    handle.play()?;
                    msg.channel_id.say(ctx, "Resumed").await?;
                }
                _ => {
                    msg.channel_id.say(ctx, "Nothing playing").await?;
                }
            }
        } else {
            msg.channel_id.say(ctx, "Nothing playing").await?;
        }
    }

    Ok(())
}
