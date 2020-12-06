use std::{convert::TryInto, result::Result as StdResult};

use serenity::{
    framework::standard::{
        macros::{check, command},
        Args, CommandOptions, CommandResult, Reason,
    },
    model::prelude::*,
    prelude::*,
    utils::parse_mention,
};

use crate::{
    consts::INSUFFICIENT_PERMISSIONS_MESSAGE,
    db::{delete_user, get_all_users_with_perm, get_user_perms, set_user_perms, UserPerm},
    state::PoolContainer,
};

async fn args_to_user(
    ctx: &Context,
    msg: &Message,
    args: &mut Args,
) -> anyhow::Result<Option<User>> {
    let mentioned_user = match args.single_quoted::<String>() {
        Ok(user) => user,
        Err(_) => {
            msg.reply_ping(
                ctx,
                "Please mention the user you would like to edit the permissions of",
            )
            .await?;
            return Ok(None);
        }
    };

    let mentioned_user = match parse_mention(mentioned_user) {
        Some(id) => id,
        None => {
            msg.reply_ping(ctx, "Not a valid mention").await?;
            return Ok(None);
        }
    };

    let user = match UserId(mentioned_user).to_user(ctx).await {
        Ok(user) => user,
        Err(_) => {
            msg.reply_ping(ctx, "Not a valid user").await?;
            return Ok(None);
        }
    };

    Ok(Some(user))
}

#[check]
#[name = "Perms"]
async fn perms_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> StdResult<(), Reason> {
    let guild = msg.guild(ctx).await.unwrap();
    let perms = guild.member_permissions(ctx, msg.author.id).await.unwrap();

    if perms.administrator() {
        Ok(())
    } else {
        let data = ctx.data.read().await;
        let pool = data.get::<PoolContainer>().unwrap();

        if let Some(perm_level) = get_user_perms(pool, guild.id.into(), msg.author.id.into())
            .await
            .unwrap()
        {
            if let UserPerm::Admin = perm_level {
                Ok(())
            } else {
                Err(Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()))
            }
        } else {
            Err(Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()))
        }
    }
}

#[command]
#[checks(Perms)]
#[sub_commands(list, set)]
async fn perms(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply_ping(ctx, "Available commands are: `perms set`, and `perms list`")
        .await?;
    Ok(())
}

#[command]
#[checks(Perms)]
#[description = "Lists the users with the selected perm"]
#[usage = "<perm level>"]
async fn list(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let level = match args.single_quoted::<String>() {
        Ok(level) => level,
        Err(_) => {
            msg.reply_ping(ctx, "Please provide a permission you would like to list the members of, availible options are: `admin`, `dj`, and `blacklist`").await?;
            return Ok(());
        }
    };

    match level.to_lowercase().as_ref() {
        "admin" => {
            list_users_with_perm(ctx, msg, UserPerm::Admin).await?;
        }
        "dj" => {
            list_users_with_perm(ctx, msg, UserPerm::DJ).await?;
        }
        "blacklist" => {
            list_users_with_perm(ctx, msg, UserPerm::Blacklisted).await?;
        }
        _ => {
            msg.reply_ping(
                ctx,
                "Not a valid permission, options are: `admin`, `dj`, and `blacklist`",
            )
            .await?;
        }
    };

    Ok(())
}

async fn list_users_with_perm(
    ctx: &Context,
    msg: &Message,
    perm_level: UserPerm,
) -> anyhow::Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let guild_id = msg.guild_id.unwrap();

    let returned_users = get_all_users_with_perm(pool, guild_id.into(), perm_level).await?;

    if returned_users.is_empty() {
        msg.channel_id
            .say(ctx, format!("No users with {:?} role", perm_level))
            .await?;
    } else {
        msg.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title(format!("Users with {:?} permission", perm_level));
                    let mut user_list = String::new();
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

#[command]
#[checks(Perms)]
#[description = "Sets a users permission to the selected perm"]
#[usage = "<mentioned user> <perm level>"]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let user = match args_to_user(ctx, msg, &mut args).await? {
        Some(user) => user,
        None => return Ok(()),
    };

    let perm_level = match args.single_quoted::<String>() {
        Ok(level) => level,
        Err(_) => {
            msg.reply_ping(ctx, "Please provide a permission you would like to set the member to, availible options are: `admin`, `dj`, `user`, and `blacklist`").await?;
            return Ok(());
        }
    };

    match perm_level.to_lowercase().as_ref() {
        "admin" => {
            set_user_perm_from_command(ctx, msg, UserPerm::Admin, user).await?;
        }
        "dj" => {
            set_user_perm_from_command(ctx, msg, UserPerm::DJ, user).await?;
        }
        "user" => {
            let data = ctx.data.read().await;
            let pool = data.get::<PoolContainer>().unwrap();

            let guild_id = msg.guild_id.unwrap();

            delete_user(pool, guild_id.into(), user.id.try_into().unwrap()).await?;

            msg.channel_id
                .say(ctx, format!("Set {}'s permission to User", user.mention()))
                .await?;
        }
        "blacklist" => {
            set_user_perm_from_command(ctx, msg, UserPerm::Blacklisted, user).await?;
        }
        _ => {
            msg.reply_ping(
                ctx,
                "Not a valid permission, options are `admin`, `dj`, `user`, and `blacklist`",
            )
            .await?;
        }
    }

    Ok(())
}

async fn set_user_perm_from_command(
    ctx: &Context,
    msg: &Message,
    perm_level: UserPerm,
    user: User,
) -> anyhow::Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let guild_id = msg.guild_id.unwrap();

    let user_perm = set_user_perms(
        pool,
        guild_id.into(),
        user.id.try_into().unwrap(),
        perm_level,
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
