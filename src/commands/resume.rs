use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use songbird::tracks::PlayMode;

use crate::checks::*;

#[command]
#[only_in(guilds)]
#[checks(Player)]
#[description = "Resumes a paused track"]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if let Some(handle) = queue.current() {
            match handle.get_info()?.await?.playing {
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
