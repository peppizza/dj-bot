use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Color,
};

use crate::{checks::*, lyrics_api::get_lyrics};

#[command]
#[checks(not_blacklisted)]
#[description = "Shows the lyrics to a song.  If no arguments are provided it will show the lyrics of the currently playing song"]
async fn lyrics(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let name_of_song = match args.remains() {
        Some(name) => name.to_string(),
        None => {
            let manager = songbird::get(ctx).await.unwrap().clone();

            if let Some(handler_lock) = manager.get(msg.guild_id.unwrap()) {
                let handler = handler_lock.lock().await;
                let queue = handler.queue();

                if let Some(handle) = queue.current() {
                    let metadata = handle.metadata();
                    let title = metadata.title.clone().unwrap();
                    let artist = metadata.artist.clone().unwrap();
                    format!("{} {}", title, artist)
                } else {
                    msg.channel_id.say(ctx, "Nothing playing").await?;
                    return Ok(());
                }
            } else {
                msg.channel_id.say(ctx, "Nothing playing").await?;
                return Ok(());
            }
        }
    };

    msg.channel_id
        .say(ctx, format!("Searching the lyrics for {}", name_of_song))
        .await?;

    let song_data = match get_lyrics(ctx, name_of_song.clone()).await? {
        Some(data) => data,
        None => {
            msg.channel_id
                .say(ctx, format!("Could not find lyrics for {}", name_of_song))
                .await?;
            return Ok(());
        }
    };

    let name = song_data.name;
    let artist = song_data.artist;
    let lyrics = song_data.lyrics;

    if lyrics.len() > 2048 {
        let subs = lyrics
            .as_bytes()
            .chunks(2048)
            .map(|v| unsafe { std::str::from_utf8_unchecked(v) })
            .collect::<Vec<&str>>();

        for (idx, sub) in subs.iter().enumerate() {
            msg.channel_id
                .send_message(ctx, |m| {
                    m.embed(|e| {
                        if idx == 0 {
                            e.title(format!("Lyrics for `{}` by `{}`", name, artist));
                        }

                        e.description(sub);

                        e.color(Color::DARK_GREEN);

                        e
                    })
                })
                .await?;
        }
    } else {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title(format!("Lyrics for `{}` by `{}`", name, artist));

                    e.description(lyrics);

                    e.color(Color::DARK_GREEN);

                    e
                })
            })
            .await?;
    }

    Ok(())
}
