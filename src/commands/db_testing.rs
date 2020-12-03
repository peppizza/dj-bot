use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{
    db::{delete_guild, delete_user, get_all_users_with_perm, get_user_perms, set_user_perms},
    state::PoolContainer,
};

#[command]
#[only_in(guilds)]
#[owners_only]
#[help_available(false)]
async fn get_author_perms(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let user_perms = get_user_perms(pool, msg.guild_id.unwrap().into(), msg.author.id.into()).await;

    msg.reply_ping(ctx, format!("{:?}", user_perms)).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[owners_only]
#[help_available(false)]
async fn set_author_perms(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let perm_level = args.single::<i16>()?;

    let perm_level = set_user_perms(
        pool,
        msg.guild_id.unwrap().into(),
        msg.author.id.into(),
        perm_level.into(),
    )
    .await;

    msg.reply_ping(ctx, format!("{:?}", perm_level)).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[owners_only]
#[help_available(false)]
async fn get_perms_in_guild(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let perm_level = args.single::<i16>()?;

    let list_of_users =
        get_all_users_with_perm(pool, msg.guild_id.unwrap().into(), perm_level.into()).await;

    msg.reply_ping(ctx, format!("{:?}", list_of_users)).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[owners_only]
#[help_available(false)]
async fn delete_author(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let returned_user = delete_user(pool, msg.guild_id.unwrap().into(), msg.author.id.into()).await;

    msg.reply_ping(ctx, format!("{:?}", returned_user)).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[owners_only]
#[help_available(false)]
async fn delete_current_guild(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let returned_guild = delete_guild(pool, msg.guild_id.unwrap().into()).await;

    msg.reply_ping(ctx, format!("{:?}", returned_guild)).await?;

    Ok(())
}
