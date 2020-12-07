use std::{convert::TryInto, result::Result as StdResult};

use serenity::{
    framework::standard::{macros::check, Args, CommandOptions, Reason},
    model::prelude::*,
    prelude::*,
};

use crate::{
    consts::INSUFFICIENT_PERMISSIONS_MESSAGE,
    db::{get_user_perms, UserPerm},
    state::{PoolContainer, SongAuthorContainer},
};

#[check]
#[name = "Player"]
pub async fn player_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    options: &CommandOptions,
) -> StdResult<(), Reason> {
    let guild = msg.guild(ctx).await.unwrap();
    let perms = guild.member_permissions(ctx, msg.author.id).await.unwrap();

    if perms.administrator() {
        Ok(())
    } else {
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
        match options.names[0] {
            "join" => map_check_result(allow_everyone_not_blacklisted(ctx, msg).await),
            "leave" => map_check_result(allow_everyone_not_blacklisted(ctx, msg).await),
            "loop" => map_check_result(allow_only_dj(ctx, msg).await),
            "mute" => map_check_result(allow_only_dj(ctx, msg).await),
            "now_playing" => map_check_result(allow_everyone_not_blacklisted(ctx, msg).await),
            "pause" => map_check_result(allow_only_dj(ctx, msg).await),
            "play" => map_check_result(allow_everyone_not_blacklisted(ctx, msg).await),
            "queue" => map_check_result(allow_everyone_not_blacklisted(ctx, msg).await),
            "remove" => map_check_result(allow_only_dj(ctx, msg).await),
            "restart" => map_check_result(allow_only_dj(ctx, msg).await),
            "resume" => map_check_result(allow_only_dj(ctx, msg).await),
            "skip" => map_check_result(allow_author_or_dj(ctx, msg).await),
            "stop" => map_check_result(allow_only_dj(ctx, msg).await),
            "volume" => map_check_result(allow_only_dj(ctx, msg).await),
            "shuffle" => map_check_result(allow_only_dj(ctx, msg).await),
            _ => Ok(()),
        }
    }
}

fn map_check_result(result: anyhow::Result<()>) -> StdResult<(), Reason> {
    if let Err(e) = result {
        match e.downcast::<Reason>() {
            Ok(reason) => Err(reason),
            Err(e) => Err(Reason::Log(format!("{:?}", e))),
        }
    } else {
        Ok(())
    }
}

async fn allow_everyone_not_blacklisted(ctx: &Context, msg: &Message) -> anyhow::Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let user_perm = match get_user_perms(
        pool,
        msg.guild_id.unwrap().into(),
        msg.author.id.try_into().unwrap(),
    )
    .await?
    {
        Some(perm) => perm,
        None => return Ok(()),
    };

    if user_perm != UserPerm::Blacklisted {
        Ok(())
    } else {
        Err(Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()).into())
    }
}

async fn allow_only_dj(ctx: &Context, msg: &Message) -> anyhow::Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<PoolContainer>().unwrap();

    let user_perm = match get_user_perms(
        pool,
        msg.guild_id.unwrap().into(),
        msg.author.id.try_into().unwrap(),
    )
    .await?
    {
        Some(perm) => perm,
        None => return Err(Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()).into()),
    };

    if user_perm >= UserPerm::DJ {
        Ok(())
    } else {
        Err(Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()).into())
    }
}

async fn allow_author_or_dj(ctx: &Context, msg: &Message) -> anyhow::Result<()> {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if let Some(handle) = queue.current() {
            let data = ctx.data.read().await;
            let author_container_lock = data.get::<SongAuthorContainer>().unwrap().clone();
            let author_container = author_container_lock.read().await;
            let author = author_container.get(&handle.uuid()).unwrap();

            if msg.author.id == *author {
                Ok(())
            } else {
                let pool = data.get::<PoolContainer>().unwrap();
                let user_perm =
                    match get_user_perms(pool, guild_id.into(), msg.author.id.try_into().unwrap())
                        .await?
                    {
                        Some(perm) => perm,
                        None => {
                            return Err(
                                Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()).into()
                            )
                        }
                    };

                if user_perm >= UserPerm::DJ {
                    Ok(())
                } else {
                    Err(Reason::User(INSUFFICIENT_PERMISSIONS_MESSAGE.to_string()).into())
                }
            }
        } else {
            Err(Reason::User("There is no track currently playing".to_string()).into())
        }
    } else {
        Err(Reason::User("You are not in a voice channel".to_string()).into())
    }
}
