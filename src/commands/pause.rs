use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use songbird::tracks::PlayMode;

use super::consts::SONGBIRD_EXPECT;

#[command]
#[only_in(guilds)]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.expect(SONGBIRD_EXPECT).clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if let Some(handle) = queue.current() {
            match handle.get_info()?.await?.playing {
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
