use std::time::Duration;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Color,
};

use songbird::Event;

use uuid::Uuid;

use crate::{
    checks::*,
    data::ReqwestClientContainer,
    playlists::{get_list_of_spotify_tracks, get_list_of_urls, get_ytdl_metadata},
    queue::{get_queue_from_ctx_and_guild_id, QueueMap, QueuedTrack},
    voice_events::ChannelIdleChecker,
};

#[command]
#[aliases("p")]
#[checks(dj_only)]
#[description = "Adds a new song to the queue, can either be the name of a song, or a link to it"]
#[usage = "<name or url of song>"]
#[bucket = "global"]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let url = match args.remains() {
        Some(url) => url.to_string(),
        None => {
            msg.channel_id
                .say(
                    ctx,
                    "Must provide a url to video or audio, or the name of a song",
                )
                .await?;

            return Ok(());
        }
    };

    let guild = msg.guild(ctx).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await.unwrap().clone();
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
                let mut handler = handler_lock.lock().await;
                let data = ctx.data.read().await;
                let queue_container = data.get::<QueueMap>().unwrap().clone();
                let queue = queue_container.entry(guild_id).or_default();

                handler.add_global_event(
                    Event::Periodic(Duration::from_secs(60), None),
                    ChannelIdleChecker {
                        handler_lock: handler_lock.clone(),
                        elapsed: Default::default(),
                        guild_id,
                        chan_id: msg.channel_id,
                        http: ctx.http.clone(),
                        cache: ctx.cache.clone(),
                        queue: queue.clone(),
                    },
                );

                msg.channel_id
                    .say(ctx, format!("Joined {}", connect_to.mention()))
                    .await?;
            } else {
                msg.channel_id
                    .say(ctx, "There was an error joining the channel")
                    .await?;
                return Ok(());
            }

            handler_lock
        }
    };

    if url.starts_with("http") {
        if url.contains("list=") {
            let mut reply_msg = msg
                .channel_id
                .send_message(ctx, |m| m.embed(|e| e.title("Downloading playlist...")))
                .await?;

            let urls = get_list_of_urls(&url).await?;

            for url in urls {
                let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;

                let guild = msg.guild(ctx).await.unwrap();

                if guild
                    .voice_states
                    .get(&ctx.cache.current_user_id().await)
                    .is_none()
                {
                    queue.stop();
                    break;
                }
                queue
                    .add(
                        QueuedTrack {
                            name: url.title,
                            uuid: Uuid::new_v4(),
                        },
                        handler_lock.clone(),
                        msg.channel_id,
                        ctx.http.clone(),
                    )
                    .await?;
            }

            reply_msg
                .edit(ctx, |m| {
                    m.embed(|e| e.title("Finished downloading playlist"))
                })
                .await?;
        } else if url.starts_with("https://open.spotify.com/playlist/") {
            let mut reply_msg = msg
                .channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| e.title("Downloading spotify playlist..."))
                })
                .await?;

            let url = url
                .strip_prefix("https://open.spotify.com/playlist/")
                .unwrap();

            let url = url.split('?').take(1).collect::<Vec<_>>()[0];

            let tracks = {
                let data = ctx.data.read().await;
                let client = data.get::<ReqwestClientContainer>().unwrap().clone();
                get_list_of_spotify_tracks(client, url).await?
            };

            for item in tracks.items {
                let track = item.track;
                let formatted_search = format!("{} {}", track.name, track.artists[0].name);

                let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;

                let guild = msg.guild(ctx).await.unwrap();

                if guild
                    .voice_states
                    .get(&ctx.cache.current_user_id().await)
                    .is_none()
                {
                    queue.stop();
                    break;
                }
                queue
                    .add(
                        QueuedTrack {
                            name: formatted_search,
                            uuid: Uuid::new_v4(),
                        },
                        handler_lock.clone(),
                        msg.channel_id,
                        ctx.http.clone(),
                    )
                    .await?;
            }

            reply_msg
                .edit(ctx, |m| {
                    m.embed(|e| e.title("Finished downloading spotify playlist"))
                })
                .await?;
        } else {
            let mut reply_msg = msg
                .channel_id
                .send_message(ctx, |m| m.embed(|e| e.title("Downloading song...")))
                .await?;

            let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;

            queue
                .add(
                    QueuedTrack {
                        name: url,
                        uuid: Uuid::new_v4(),
                    },
                    handler_lock,
                    msg.channel_id,
                    ctx.http.clone(),
                )
                .await?;

            reply_msg
                .edit(ctx, |m| m.embed(|e| e.title("Added song to queue")))
                .await?;
        }
    } else {
        let mut reply_msg = msg
            .channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Searching for song...")
                        .description(format!("Searching for `{}`", url))
                })
            })
            .await?;

        let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;

        if queue.current().lock().is_none() {
            queue
                .add(
                    QueuedTrack {
                        name: url,
                        uuid: Uuid::new_v4(),
                    },
                    handler_lock,
                    msg.channel_id,
                    ctx.http.clone(),
                )
                .await?;

            let track_metadata = queue.current().lock().as_ref().unwrap().metadata().clone();

            reply_msg
                .edit(ctx, |m| {
                    m.embed(|e| {
                        let title = track_metadata.title.unwrap();
                        let artist = track_metadata.channel.unwrap();
                        let length = track_metadata.duration.unwrap();
                        let mut seconds = (length.as_secs() % 60).to_string();
                        let minutes = (length.as_secs() / 60) % 60;
                        let url = track_metadata.source_url.unwrap();
                        let hours = (length.as_secs() / 60) / 60;

                        if seconds.len() < 2 {
                            seconds = format!("0{}", seconds);
                        }

                        e.title(format!("Added song: {}", title));
                        e.fields(vec![
                            ("Title:", format!("[{}]({})", title, url), true),
                            ("Artist", artist, true),
                            (
                                "Spot in queue",
                                (queue.len()).to_string(),
                                true,
                            ),
                            ("Length", format!("{}:{}:{}", hours, minutes, seconds), true),
                        ]);

                        e.footer(|f| {
                            f.icon_url("https://avatars0.githubusercontent.com/u/35662205?s=460&u=a154620c136da5ad4acc9c473864cc6349a4e874&v=4");
                            f.text("If you like my work consider donating, ~donate");

                            f
                        });

                        e.color(Color::DARK_GREEN);

                        e
                    })
                })
                .await?;
        } else {
            queue
                .add(
                    QueuedTrack {
                        name: url.clone(),
                        uuid: Uuid::new_v4(),
                    },
                    handler_lock,
                    msg.channel_id,
                    ctx.http.clone(),
                )
                .await?;
            let track_metadata = get_ytdl_metadata(&url).await?;

            reply_msg
        .edit(ctx, |m| {
            m.embed(|e| {
                let title = track_metadata.title;
                let artist = track_metadata.uploader;
                let length = Duration::from_secs(track_metadata.duration as u64);
                let mut seconds = (length.as_secs() % 60).to_string();
                let minutes = (length.as_secs() / 60) % 60;
                let url = track_metadata.webpage_url;
                let hours = (length.as_secs() / 60) / 60;

                if seconds.len() < 2 {
                    seconds = format!("0{}", seconds);
                }

                e.title(format!("Added song: {}", title));
                e.fields(vec![
                    ("Title:", format!("[{}]({})", title, url), true),
                    ("Artist", artist, true),
                    (
                        "Spot in queue",
                        (queue.len()).to_string(),
                        true,
                    ),
                    ("Length", format!("{}:{}:{}", hours, minutes, seconds), true),
                ]);

                e.footer(|f| {
                    f.icon_url("https://avatars0.githubusercontent.com/u/35662205?s=460&u=a154620c136da5ad4acc9c473864cc6349a4e874&v=4");
                    f.text("If you like my work consider donating, ~donate");

                    f
                });

                e.color(Color::DARK_GREEN);

                e
            })
        })
        .await?;
        }
    };

    Ok(())
}

#[command]
#[help_available(false)]
async fn donate(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx, "https://ko-fi.com/peppizza")
        .await?;
    Ok(())
}
