use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use super::consts::SONGBIRD_EXPECT;

#[command]
#[only_in(guilds)]
async fn volume(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let new_volume = match args.single_quoted::<i32>() {
        Ok(vol) => vol,
        Err(_) => {
            msg.reply(ctx, "Please enter a valid number").await?;

            return Ok(());
        }
    };

    if new_volume < 0 || new_volume > 100 {
        msg.reply(ctx, "Please select a value from 0 to 100")
            .await?;
        return Ok(());
    }

    let new_volume: f32 = new_volume as f32 / 100f32;

    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.expect(SONGBIRD_EXPECT).clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if let Some(handle) = queue.current() {
            handle.set_volume(new_volume)?;
        } else {
            msg.reply(ctx, "Nothing playing").await?;
        }
    } else {
        msg.reply(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}
