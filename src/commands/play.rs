use crate::state::{TrackEndNotifier, VoiceQueueManager};

use super::consts::{SONGBIRD_EXPECT, VOICEQUEUEMANAGER_NOT_FOUND};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use songbird::input;

use tracing::error;

#[command]
#[only_in(guilds)]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single_quoted::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id
                .say(
                    ctx,
                    "Must provide a url to video or audio, or the name of a song",
                )
                .await?;

            return Ok(());
        }
    };

    let typing = ctx.http.start_typing(msg.channel_id.0)?;

    let source = if url.starts_with("http") {
        match input::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                error!("Err starting source: {:?}", why);

                msg.channel_id.say(ctx, "Error sourcing ffmpeg").await?;

                return Ok(());
            }
        }
    } else {
        match input::ytdl_search(&url).await {
            Ok(source) => source,
            Err(why) => {
                error!("Err starting source: {:?}", why);

                msg.channel_id.say(ctx, "Error sourcing ffmpeg").await?;

                return Ok(());
            }
        }
    };

    typing.stop();

    let metadata = source.metadata.clone();

    let guild = msg.guild(ctx).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await.expect(SONGBIRD_EXPECT).clone();
    let queues_lock = ctx
        .data
        .read()
        .await
        .get::<VoiceQueueManager>()
        .cloned()
        .expect(VOICEQUEUEMANAGER_NOT_FOUND);

    let mut track_queues = queues_lock.lock().await;

    let handler_lock = {
        let is_in_channel = manager.get(guild_id);

        if let Some(handler_lock) = is_in_channel {
            handler_lock
        } else {
            let channel_id = guild
                .voice_states
                .get(&msg.author.id)
                .and_then(|voice_state| voice_state.channel_id);

            let connect_to = match channel_id {
                Some(c) => c,
                None => {
                    msg.channel_id
                        .say(ctx, "Not in a channel to join into")
                        .await?;

                    return Ok(());
                }
            };

            let (handler_lock, success) = manager.join(guild_id, connect_to).await;
            if success.is_ok() {
                let mut handle = handler_lock.lock().await;

                let send_http = ctx.http.clone();

                handle.add_global_event(
                    songbird::Event::Track(songbird::TrackEvent::End),
                    TrackEndNotifier {
                        chan_id: msg.channel_id,
                        http: send_http,
                    },
                );

                msg.channel_id
                    .say(ctx, format!("Joined channel, {}", connect_to.mention()))
                    .await?;
            }

            handler_lock
        }
    };

    let mut handler = handler_lock.lock().await;

    let queue = track_queues.entry(guild_id).or_default();

    queue.add_source(source, &mut handler);

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                let title = metadata.title.unwrap();
                let artist = metadata.artist.unwrap();
                let length = metadata.duration.unwrap();
                let seconds = length.as_secs() % 60;
                let minutes = (length.as_secs() / 60) % 60;

                e.title(format!("Added song: {}", title));
                e.fields(vec![
                    ("Title:", title, true),
                    ("Artist", artist, true),
                    ("Spot in queue", queue.len().to_string(), true),
                    ("Length", format!("{}:{}", minutes, seconds), true),
                ]);

                e
            })
        })
        .await?;

    Ok(())
}
