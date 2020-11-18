use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[only_in(guilds)]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let index = args.single_quoted::<usize>()?;

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if !queue.is_empty() {
            if index == 1 {
                queue.skip()?;
                msg.channel_id.say(ctx, "Skipped the song").await?;
            } else if queue.dequeue(index - 1).is_none() {
                msg.reply(ctx, format!("There is no song at {}", index))
                    .await?;
            } else {
                msg.channel_id
                    .say(ctx, format!("Removed the song at {}", index))
                    .await?;
            }
        } else {
            msg.channel_id.say(ctx, "The queue is empty").await?;
        }
    }

    Ok(())
}
