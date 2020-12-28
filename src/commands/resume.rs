use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use songbird::tracks::PlayMode;

use crate::{checks::*, queue::get_queue_from_ctx_and_guild_id};

#[command]
#[checks(dj_only)]
#[description = "Resumes a paused track"]
#[aliases("unpause")]
#[bucket = "global"]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;

        if let Some(handle) = queue.current() {
            match handle.get_info().await?.playing {
                PlayMode::Pause => {
                    handle.play()?;
                    msg.channel_id.say(ctx, "Resumed").await?;
                }
                PlayMode::Play => {
                    msg.channel_id.say(ctx, "Already playing").await?;
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
