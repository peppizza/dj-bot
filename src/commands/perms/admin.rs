use std::convert::TryInto;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::{
    db::{get_all_users_with_perm, set_user_perms, UserPerm},
    state::PoolContainer,
};

use super::util::args_to_user;

#[command]
#[only_in(guilds)]
#[sub_commands(add, del, list)]
async fn admin(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(
        ctx,
        "The available commands are `admin add`, `admin del`, and `admin list`",
    )
    .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let user = match args_to_user(ctx, msg, args).await? {
        Some(user) => user,
        None => return Ok(()),
    };

    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let guild_id = msg.guild_id.unwrap();

    let user_perm = set_user_perms(
        pool,
        guild_id.into(),
        user.id.try_into().unwrap(),
        UserPerm::Admin,
    )
    .await?;

    msg.channel_id
        .say(
            ctx,
            format!("Set {}'s permission to {:?}", user.mention(), user_perm),
        )
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn del(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let user = match args_to_user(ctx, msg, args).await? {
        Some(user) => user,
        None => return Ok(()),
    };

    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let guild_id = msg.guild_id.unwrap();

    let user_perm = set_user_perms(
        pool,
        guild_id.into(),
        user.id.try_into().unwrap(),
        UserPerm::User,
    )
    .await?;

    msg.channel_id
        .say(
            ctx,
            format!("Set {}'s permission to {:?}", user.mention(), user_perm),
        )
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let guild_id = msg.guild_id.unwrap();

    let returned_users = get_all_users_with_perm(pool, guild_id.into(), UserPerm::Admin).await?;

    if returned_users.is_empty() {
        msg.channel_id.say(ctx, "No users with Admin role").await?;
    } else {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title("Users with admin permission");
                    let mut user_list = "".to_string();
                    for user in returned_users {
                        let user = UserId(user.user_id.try_into().unwrap());
                        user_list.push_str(&format!("{}\n", user.mention()));
                    }

                    e.description(user_list);

                    e
                })
            })
            .await?;
    }

    Ok(())
}
