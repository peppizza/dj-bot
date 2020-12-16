use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{
    checks::*,
    data::DjOnlyContainer,
    dj_only_store::{check_if_guild_in_store, delete_guild_from_store, insert_guild_into_store},
};

#[command]
#[checks(admin_only)]
#[description = "Enables/Disables dj only mode"]
#[bucket = "global"]
async fn dj_only(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let redis_con = data.get::<DjOnlyContainer>().unwrap().clone();

    if check_if_guild_in_store(&redis_con, guild_id).await? {
        delete_guild_from_store(&redis_con, guild_id).await?;
        msg.channel_id.say(ctx, "Disabled dj only mode").await?;
    } else {
        insert_guild_into_store(&redis_con, guild_id).await?;
        msg.channel_id.say(ctx, "Enabled dj only mode").await?;
    }

    Ok(())
}
