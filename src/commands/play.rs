use std::time::Duration;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Color,
};

use songbird::{input::Restartable, Event};

use tracing::error;

use crate::{
    checks::*,
    data::ReqwestClientContainer,
    playlists::{get_list_of_spotify_tracks, get_list_of_urls},
    voice_events::{ChannelIdleChecker, TrackStartNotifier},
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
                handler.add_global_event(
                    Event::Periodic(Duration::from_secs(60), None),
                    ChannelIdleChecker {
                        handler_lock: handler_lock.clone(),
                        elapsed: Default::default(),
                        guild_id,
                        chan_id: msg.channel_id,
                        http: ctx.http.clone(),
                        cache: ctx.cache.clone(),
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

    let (source, mut reply_msg) = if url.starts_with("http") {
        if url.contains("list=") {
            let mut reply_msg = msg
                .channel_id
                .send_message(ctx, |m| m.embed(|e| e.title("Downloading playlist...")))
                .await?;

            let urls = get_list_of_urls(url).await?;

            for url in urls {
                let input = Restartable::ytdl(url.url.clone()).await;

                let input = match input {
                    Ok(i) => songbird::input::Input::from(i),
                    Err(e) => {
                        error!("Error starting source: {:?}", e);

                        continue;
                    }
                };

                let mut handler = handler_lock.lock().await;

                let (track, handle) = songbird::create_player(input);

                if !handler.queue().is_empty() {
                    handle.add_event(
                        Event::Delayed(Duration::from_millis(5)),
                        TrackStartNotifier {
                            chan_id: msg.channel_id,
                            http: ctx.http.clone(),
                        },
                    )?;
                }

                let guild = msg.guild(ctx).await.unwrap();

                if guild
                    .voice_states
                    .get(&ctx.cache.current_user_id().await)
                    .is_none()
                {
                    handler.queue().stop();
                    break;
                }
                handler.enqueue(track);
            }

            reply_msg
                .edit(ctx, |m| {
                    m.embed(|e| e.title("Finished downloading playlist"))
                })
                .await?;

            return Ok(());
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
                let formatted_search = format!("{}{}", track.name, track.artists[0].name);

                let input = Restartable::ytdl_search(&formatted_search).await;

                let input = match input {
                    Ok(i) => songbird::input::Input::from(i),
                    Err(e) => {
                        error!("Error starting source: {:?}", e);

                        continue;
                    }
                };

                let mut handler = handler_lock.lock().await;

                let (track, handle) = songbird::create_player(input);

                if !handler.queue().is_empty() {
                    handle.add_event(
                        Event::Delayed(Duration::from_millis(5)),
                        TrackStartNotifier {
                            chan_id: msg.channel_id,
                            http: ctx.http.clone(),
                        },
                    )?;
                }

                let guild = msg.guild(ctx).await.unwrap();

                if guild
                    .voice_states
                    .get(&ctx.cache.current_user_id().await)
                    .is_none()
                {
                    handler.queue().stop();
                    break;
                }
                handler.enqueue(track);
            }

            reply_msg
                .edit(ctx, |m| {
                    m.embed(|e| e.title("Finished downloading spotify playlist"))
                })
                .await?;

            return Ok(());
        } else {
            let mut reply_msg = msg
                .channel_id
                .send_message(ctx, |m| m.embed(|e| e.title("Downloading song...")))
                .await?;

            let source = match Restartable::ytdl(url).await {
                Ok(source) => source,
                Err(why) => {
                    error!("Err starting source: {:?}", why);

                    reply_msg
                        .edit(ctx, |m| {
                            m.content("There was a problem downloading the song")
                        })
                        .await?;

                    return Ok(());
                }
            };

            (source, reply_msg)
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

        let source = match Restartable::ytdl_search(&url).await {
            Ok(source) => source,
            Err(why) => {
                error!("Err starting source: {:?}", why);

                reply_msg
                    .edit(ctx, |m| {
                        m.embed(|e| {
                            e.title("There was a problem downloading that song, try using the direct url");

                            e.color(Color::DARK_RED);

                            e
                        })
                    })
                    .await?;

                return Ok(());
            }
        };

        (source, reply_msg)
    };

    let source = songbird::input::Input::from(source);
    let metadata = source.metadata.clone();

    let mut handler = handler_lock.lock().await;

    let (track, handle) = songbird::create_player(source);

    if !handler.queue().is_empty() {
        let send_http = ctx.http.clone();

        handle.add_event(
            Event::Delayed(Duration::from_millis(5)),
            TrackStartNotifier {
                chan_id: msg.channel_id,
                http: send_http,
            },
        )?;
    }

    handler.enqueue(track);

    reply_msg
        .edit(ctx, |m| {
            m.embed(|e| {
                let title = metadata.title.unwrap_or_default();
                let artist = metadata.artist.unwrap_or_default();
                let length = metadata.duration.unwrap_or_default();
                let mut seconds = (length.as_secs() % 60).to_string();
                let minutes = (length.as_secs() / 60) % 60;
                let url = metadata.source_url.unwrap_or_default();

                if seconds.len() < 2 {
                    seconds = format!("0{}", seconds);
                }

                e.title(format!("Added song: {}", title));
                e.fields(vec![
                    ("Title:", format!("[{}]({})", title, url), true),
                    ("Artist", artist, true),
                    (
                        "Spot in queue",
                        (handler.queue().len() - 1).to_string(),
                        true,
                    ),
                    ("Length", format!("{}:{}", minutes, seconds), true),
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
