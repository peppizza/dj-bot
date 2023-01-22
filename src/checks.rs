use std::result::Result as StdResult;

use serenity::{
    framework::standard::{macros::check, Args, CommandOptions, Reason},
    model::prelude::*,
    prelude::*,
};

use crate::{
    consts::INSUFFICIENT_PERMISSIONS_MESSAGE,
    data::{DjOnlyContainer, PoolContainer},
    db::{get_user_perms, UserPerm},
    dj_only_store::check_if_guild_in_store,
};

#[check]
#[name = "not_blacklisted"]
async fn not_blacklisted(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> StdResult<(), Reason> {
    let guild = msg.guild(ctx).await.unwrap();
    if check_if_administrator(ctx, guild, msg.author.id).await {
        Ok(())
    } else {
        check_if_already_playing(ctx, msg).await?;
        let perm_level = get_author_perm_level(ctx, msg).await?;
        if perm_level != UserPerm::Blacklisted {
            Ok(())
        } else {
            Err(Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()))
        }
    }
}

#[check]
#[name = "dj_only"]
async fn dj_only(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> StdResult<(), Reason> {
    let guild = msg.guild(ctx).await.unwrap();
    if check_if_administrator(ctx, guild, msg.author.id).await {
        Ok(())
    } else {
        check_if_already_playing(ctx, msg).await?;
        let perm_level = get_author_perm_level(ctx, msg).await?;
        if perm_level != UserPerm::Blacklisted {
            if guild_has_dj_mode_enabled(ctx, msg).await? || perm_level >= UserPerm::Dj {
                Ok(())
            } else {
                Err(Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()))
            }
        } else {
            Err(Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()))
        }
    }
}

#[check]
#[name = "admin_only"]
async fn admin_only(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> StdResult<(), Reason> {
    let guild = msg.guild(ctx).await.unwrap();

    if check_if_administrator(ctx, guild, msg.author.id).await {
        Ok(())
    } else {
        let perm_level = get_author_perm_level(ctx, msg).await?;
        if perm_level == UserPerm::Admin {
            Ok(())
        } else {
            Err(Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()))
        }
    }
}

async fn check_if_administrator(ctx: &Context, guild: Guild, author: UserId) -> bool {
    let perms = guild.member_permissions(ctx, author).await.unwrap();
    perms.administrator()
}

async fn check_if_already_playing(ctx: &Context, msg: &Message) -> StdResult<(), Reason> {
    let guild = msg.guild(ctx).await.unwrap();

    let author_channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let bot_channel_id = guild
        .voice_states
        .get(&ctx.cache.current_user_id().await)
        .and_then(|voice_state| voice_state.channel_id);

    if let Some(bot_channel_id) = bot_channel_id {
        if let Some(author_channel_id) = author_channel_id {
            if bot_channel_id != author_channel_id {
                return Err(Reason::User(
                    "Already in a different voice channel".to_string(),
                ));
            }
        }
    }

    Ok(())
}

async fn get_author_perm_level(ctx: &Context, msg: &Message) -> StdResult<UserPerm, Reason> {
    let guild_id = msg.guild_id.unwrap();
    let author_id = msg.author.id;

    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let user_perm = get_user_perms(pool, guild_id.into(), author_id.into())
        .await
        .map_err(|e| Reason::Log(format!("{e:?}")))?;

    if let Some(level) = user_perm {
        Ok(level)
    } else {
        Ok(UserPerm::User)
    }
}

async fn guild_has_dj_mode_enabled(ctx: &Context, msg: &Message) -> StdResult<bool, Reason> {
    let data = ctx.data.read().await;
    let redis_con = data.get::<DjOnlyContainer>().unwrap().clone();
    if !check_if_guild_in_store(redis_con, msg.guild_id.unwrap())
        .await
        .map_err(|e| Reason::Log(format!("{e:?}")))?
    {
        Ok(true)
    } else {
        Ok(false)
    }
}
