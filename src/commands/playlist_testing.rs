use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use futures_util::{pin_mut, StreamExt};

use crate::yt_playlist_stream::{download_playlist, get_list_of_urls};

#[command]
#[owners_only]
#[help_available(false)]
async fn list_of_urls(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = args.single::<String>()?;

    let urls = get_list_of_urls(url).await;

    msg.reply(ctx, format!("{:?}", urls)).await?;

    Ok(())
}

#[command]
#[owners_only]
#[help_available(false)]
async fn stream_playlist(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = args.single::<String>()?;

    let urls = get_list_of_urls(url).await?;

    let stream = download_playlist(urls);

    pin_mut!(stream);

    while let Some(input) = stream.next().await {
        msg.reply(ctx, format!("{:?}", input)).await?;
    }

    Ok(())
}
