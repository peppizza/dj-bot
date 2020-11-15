use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::state::VoiceQueueManager;

use super::consts::VOICEQUEUEMANAGER_NOT_FOUND;

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

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

        msg.channel_id.say(ctx, "Queue cleared.").await?;
    } else {
        msg.channel_id
            .say(ctx, "Not in a voice channel to play in")
            .await?;
    }

    Ok(())
}
