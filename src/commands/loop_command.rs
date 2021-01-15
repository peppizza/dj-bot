use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use songbird::tracks::LoopState;

use crate::{checks::*, queue::get_queue_from_ctx_and_guild_id};

#[command("loop")]
#[checks(dj_only)]
#[description = "Enables/Disables a loop for the current track"]
#[bucket = "global"]
async fn loop_command(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;
        let current = { queue.current().lock().clone() };

        if let Some(handle) = current {
            if let LoopState::Infinite = handle.get_info().await?.loops {
                handle.disable_loop()?;

                msg.channel_id.say(ctx, "Disabled loop").await?;
            } else {
                handle.enable_loop()?;

                msg.channel_id.say(ctx, "Enabled loop").await?;
            }
        }
    } else {
        msg.reply_ping(ctx, "Not in a voice channel to loop in")
            .await?;
    }

    Ok(())
}
