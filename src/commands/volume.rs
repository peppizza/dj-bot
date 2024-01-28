use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{
    checks::*,
    queue::{get_queue_from_ctx_and_guild_id, QueueMap},
};

#[command]
#[aliases("vol")]
#[checks(dj_only)]
#[description = "Shows the current volume of the track, or sets the volume of the track if an argument is supplied"]
#[usage = "to see the current volume | volume <number 0-100> to set the volume"]
#[bucket = "global"]
async fn volume(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let new_volume = match args.single_quoted::<i32>() {
        Ok(vol) => vol,
        Err(_) => {
            let manager = songbird::get(ctx).await.unwrap().clone();

            if manager.get(guild_id).is_some() {
                let data = ctx.data.read().await;
                let queue_container = data.get::<QueueMap>().unwrap().clone();
                let queue = queue_container.get(&guild_id).unwrap();

                let current = { queue.current().lock().clone() };

                if let Some(handle) = current {
                    let mut current_volume = handle.get_info().await?.volume * 100f32;
                    current_volume = current_volume.round();

                    msg.channel_id
                        .say(ctx, format!("The current volume is {current_volume}"))
                        .await?;
                } else {
                    msg.reply_ping(ctx, "Nothing playing").await?;
                }
            } else {
                msg.reply_ping(ctx, "Not in a voice channel").await?;
            }

            return Ok(());
        }
    };

    if !(0..=100).contains(&new_volume) {
        msg.reply_ping(ctx, "Please select a value from 0 to 100")
            .await?;
        return Ok(());
    }

    let new_volume: f32 = new_volume as f32 / 100f32;

    let manager = songbird::get(ctx).await.unwrap().clone();

    if manager.get(guild_id).is_some() {
        let queue = get_queue_from_ctx_and_guild_id(ctx, guild_id).await;

        let current = { queue.current().lock().clone() };

        if let Some(handle) = current {
            handle.set_volume(new_volume)?;
        } else {
            msg.reply_ping(ctx, "Nothing playing").await?;
        }
    } else {
        msg.reply_ping(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}
