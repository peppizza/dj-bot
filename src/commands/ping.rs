use serenity::{
    client::bridge::gateway::ShardId,
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::ShardManagerContainer;

#[command]
#[description = "Shows the bots current latency"]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = data.get::<ShardManagerContainer>().unwrap().clone();

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply_ping(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    if let Some(latency) = runner.latency {
        msg.channel_id
            .say(ctx, &format!("Latency: {}ms", latency.as_millis()))
            .await?;
    } else {
        msg.channel_id
            .say(ctx, "Discord has not sent us the latency yet")
            .await?;
    }

    Ok(())
}
