use std::sync::Arc;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Color,
};

use songbird::input::Restartable;

use tracing::error;

use crate::{
    checks::*,
    state::{MetadataWithAuthor, SongMetadataContainer},
};

#[command]
#[only_in(guilds)]
#[aliases("p")]
#[checks(Player)]
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

            let (handler_lock, _) = manager.join(guild_id, connect_to).await;

            handler_lock
        }
    };

    let (source, mut reply_msg) = if url.starts_with("http") {
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
                        m.content("There was a problem searching for the song")
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

    let (track, _) = songbird::create_player(source);

    let uuid = track.uuid();

    let send_metadata = metadata.clone();

    {
        let data = ctx.data.read().await;
        let metadata_container_lock = data.get::<SongMetadataContainer>().unwrap().clone();
        let mut metadata_container = metadata_container_lock.write().await;

        metadata_container.insert(
            uuid,
            MetadataWithAuthor {
                metadata: Arc::new(send_metadata),
                author: msg.author.id,
            },
        );
    }

    handler.enqueue(track);

    reply_msg
        .edit(ctx, |m| {
            m.embed(|e| {
                let title = metadata.title.unwrap();
                let artist = metadata.artist.unwrap();
                let length = metadata.duration.unwrap();
                let mut seconds = (length.as_secs() % 60).to_string();
                let minutes = (length.as_secs() / 60) % 60;

                if seconds.len() < 2 {
                    seconds = format!("0{}", seconds);
                }

                e.title(format!("Added song: {}", title));
                e.fields(vec![
                    ("Title:", title, true),
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
                    f.text("If you like my work consider supporting me on ko-fi: https://ko-fi.com/peppizza");

                    f
                });

                e.color(Color::DARK_GREEN);

                e
            })
        })
        .await?;

    Ok(())
}
