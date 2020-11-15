use crate::state::VoiceQueueManager;

use super::consts::{SONGBIRD_EXPECT, VOICEQUEUEMANAGER_NOT_FOUND};

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.expect(SONGBIRD_EXPECT).clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id).await?;

        let queues_lock = ctx
            .data
            .read()
            .await
            .get::<VoiceQueueManager>()
            .cloned()
            .expect(VOICEQUEUEMANAGER_NOT_FOUND);

        let mut track_queues = queues_lock.lock().await;

        if let Some(queue) = track_queues.get_mut(&guild_id) {
            queue.stop()?;
        }

        msg.channel_id.say(ctx, "Left voice channel").await?;
    } else {
        msg.reply(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}
