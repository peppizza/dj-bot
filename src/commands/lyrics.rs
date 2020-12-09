use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Color,
};

use crate::{checks::*, lyrics_api::get_lyrics};

#[command]
#[checks(not_blacklisted)]
#[description = "Shows the lyrics to a song"]
async fn lyrics(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let name_of_song = match args.remains() {
        Some(name) => name,
        None => {
            msg.channel_id
                .say(
                    ctx,
                    "Please supply the name of a song you want the lyrics to",
                )
                .await?;
            return Ok(());
        }
    };

    msg.channel_id
        .say(ctx, format!("Searching the lyrics for {}", name_of_song))
        .await?;

    let song_data = get_lyrics(ctx, name_of_song.to_string()).await?;

    let name = song_data.name;
    let artist = song_data.artist;
    let lyrics = song_data.lyrics;

    if lyrics.len() > 2048 {
        let subs = lyrics
            .as_bytes()
            .chunks(2048)
            .map(std::str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()
            .unwrap();

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
