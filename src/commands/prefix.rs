use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{
    checks::*,
    data::{PoolContainer, PrefixCache},
    db::{delete_guild_prefix, get_guild_prefix, set_guild_prefix},
};

#[command]
#[checks(admin_only)]
#[description = "Shows or changes this servers prefix"]
#[bucket = "global"]
#[sub_commands(set)]
async fn prefix(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();
    let prefix_cache = data.get::<PrefixCache>().unwrap().clone();

    let guild_id = msg.guild_id.unwrap();

    let prefix = get_guild_prefix(pool, prefix_cache, guild_id.into()).await?;

    if prefix.len() > 5 {
        msg.channel_id
            .say(ctx, "The prefix has to be under 5 characters")
            .await?;
    }

    msg.channel_id
        .say(ctx, format!("The current prefix is {prefix}"))
        .await?;

    Ok(())
}

#[command]
#[checks(admin_only)]
#[description = "Changes this servers prefix"]
#[bucket = "global"]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let prefix = match args.single_quoted::<String>() {
        Ok(prefix) => prefix,
        Err(_) => {
            msg.reply_ping(ctx, "Please include the prefix you would like to set it to")
                .await?;
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();
    let prefix_cache = data.get::<PrefixCache>().unwrap().clone();

    let guild_id = msg.guild_id.unwrap();

    if prefix.len() > 5 {
        msg.reply_ping(ctx, "The prefix has to be under 5 characters")
            .await?;
        return Ok(());
    }

    if &prefix == "~" {
        delete_guild_prefix(pool, prefix_cache, guild_id.into()).await?;
        msg.channel_id.say(ctx, "Reset the servers prefix").await?;
        return Ok(());
    }

    let prefix = set_guild_prefix(pool, prefix_cache, guild_id.into(), &prefix).await?;

    msg.channel_id
        .say(ctx, format!("Set the guilds prefix to {}", prefix.prefix))
        .await?;

    Ok(())
}
