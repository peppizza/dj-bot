use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use songbird::tracks::LoopState;

use super::consts::SONGBIRD_EXPECT;

#[command("loop")]
#[only_in(guilds)]
async fn loop_command(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.expect(SONGBIRD_EXPECT).clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if let Some(handle) = queue.current() {
            if let LoopState::Infinite = handle.get_info()?.await?.loops {
                if let Err(why) = handle.disable_loop() {
                    tracing::error!("{:?}, {}", why, why);
                }

                msg.channel_id.say(ctx, "Disabled loop").await?;
            } else {
                if let Err(why) = handle.enable_loop() {
                    tracing::error!("{:?}, {}", why, why);
                }

                msg.channel_id.say(ctx, "Enabled loop").await?;
            }
        }
    } else {
        msg.reply(ctx, "Not in a voice channel to loop in").await?;
    }

    Ok(())
}
