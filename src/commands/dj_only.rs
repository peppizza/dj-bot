use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{checks::*, data::DjOnlyContainer};

#[command]
#[checks(admin_only)]
#[description = "Enables/Disables dj only mode"]
#[bucket = "global"]
async fn dj_only(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let data = ctx.data.read().await;
    let dj_only_container_lock = data.get::<DjOnlyContainer>().unwrap().clone();
    let mut dj_only_container = dj_only_container_lock.write().await;

    if let Some(id) = dj_only_container.get(&guild_id).cloned() {
        dj_only_container.remove(&id);
        msg.channel_id.say(ctx, "Disabled dj only mode").await?;
    } else {
        dj_only_container.insert(guild_id);
        msg.channel_id.say(ctx, "Enabled dj only mode").await?;
    }

    Ok(())
}
