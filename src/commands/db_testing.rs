use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{
    db::{get_all_users_with_perm, get_user_perms, set_user_perms},
    state::PoolContainer,
};

#[command]
#[only_in(guilds)]
#[owners_only]
async fn get_author_perms(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let user_perms = get_user_perms(pool, msg.guild_id.unwrap().into(), msg.author.id.into()).await;

    msg.reply(ctx, format!("{:?}", user_perms)).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[owners_only]
async fn set_author_perms(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let perm_level = args.single::<i16>()?;

    let perm_level = set_user_perms(
        pool,
        msg.guild_id.unwrap().into(),
        msg.author.id.into(),
        perm_level,
    )
    .await;

    msg.reply(ctx, format!("{:?}", perm_level)).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[owners_only]
async fn get_perms_in_guild(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let perm_level = args.single::<i16>()?;

    let list_of_users =
        get_all_users_with_perm(pool, msg.guild_id.unwrap().into(), perm_level).await?;

    msg.reply(ctx, format!("{:?}", list_of_users)).await?;

    Ok(())
}
