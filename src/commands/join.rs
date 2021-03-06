use std::time::Duration;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use songbird::Event;

use crate::{checks::*, queue::QueueMap, voice_events::ChannelIdleChecker};

#[command]
#[checks(dj_only)]
#[description = "Makes the bot join the voice channel you are in"]
#[bucket = "global"]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply_ping(ctx, "Not in a voice channel").await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        msg.channel_id
            .say(ctx, "Already in a voice channel")
            .await?;
        return Ok(());
    }

    let (handler_lock, success) = manager.join(guild_id, connect_to).await;

    if success.is_ok() {
        let mut handler = handler_lock.lock().await;
        let data = ctx.data.read().await;
        let queue_container = data.get::<QueueMap>().unwrap().clone();
        let queue = queue_container.entry(guild_id).or_default();

        handler.add_global_event(
            Event::Periodic(Duration::from_secs(60), None),
            ChannelIdleChecker {
                handler_lock: handler_lock.clone(),
                elapsed: Default::default(),
                chan_id: msg.channel_id,
                guild_id,
                http: ctx.http.clone(),
                cache: ctx.cache.clone(),
                queue: queue.clone(),
            },
        );

        msg.channel_id
            .say(ctx, format!("Joined {}", connect_to.mention()))
            .await?;
    } else {
        msg.channel_id.say(ctx, "Error joining the channel").await?;
    }

    Ok(())
}
