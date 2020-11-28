use std::convert::TryInto;

use serenity::{
    framework::standard::{macros::check, Args, CheckResult, CommandOptions},
    model::prelude::*,
    prelude::*,
};

use crate::{
    consts::INSUFFICIENT_PERMISSIONS_MESSAGE,
    db::{get_user_perms, UserPerm},
    state::{PoolContainer, SongMetadataContainer},
};

#[check]
#[name = "Player"]
pub async fn player_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    options: &CommandOptions,
) -> CheckResult {
    let guild = msg.guild(ctx).await.unwrap();
    let perms = guild.member_permissions(ctx, msg.author.id).await.unwrap();

    if perms.administrator() {
        CheckResult::Success
    } else {
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
            _ => CheckResult::Success,
        }
    }
}

fn map_check_result(result: anyhow::Result<CheckResult>) -> CheckResult {
    match result {
        Ok(result) => result,
        Err(e) => CheckResult::new_log(e),
    }
}

async fn allow_everyone_not_blacklisted(
    ctx: &Context,
    msg: &Message,
) -> anyhow::Result<CheckResult> {
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
        None => return Ok(CheckResult::Success),
    };

    if user_perm != UserPerm::Blacklisted {
        Ok(CheckResult::Success)
    } else {
        Ok(CheckResult::new_user(INSUFFICIENT_PERMISSIONS_MESSAGE))
    }
}

async fn allow_only_dj(ctx: &Context, msg: &Message) -> anyhow::Result<CheckResult> {
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
        None => return Ok(CheckResult::new_user(INSUFFICIENT_PERMISSIONS_MESSAGE)),
    };

    if user_perm >= UserPerm::DJ {
        Ok(CheckResult::Success)
    } else {
        Ok(CheckResult::new_user(INSUFFICIENT_PERMISSIONS_MESSAGE))
    }
}

async fn allow_author_or_dj(ctx: &Context, msg: &Message) -> anyhow::Result<CheckResult> {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if let Some(handle) = queue.current() {
            let data = ctx.data.read().await;
            let metadata_container_lock = data.get::<SongMetadataContainer>().unwrap().clone();
            let metadata_container = metadata_container_lock.read().await;

            let metadata = metadata_container.get(&handle.uuid()).unwrap();
            let author = metadata.author;

            if msg.author.id == author {
                Ok(CheckResult::Success)
            } else {
                let pool = data.get::<PoolContainer>().unwrap();
                let user_perm =
                    match get_user_perms(pool, guild_id.into(), msg.author.id.try_into().unwrap())
                        .await?
                    {
                        Some(perm) => perm,
                        None => return Ok(CheckResult::new_user(INSUFFICIENT_PERMISSIONS_MESSAGE)),
                    };

                if user_perm >= UserPerm::DJ {
                    Ok(CheckResult::Success)
                } else {
                    Ok(CheckResult::new_user(INSUFFICIENT_PERMISSIONS_MESSAGE))
                }
            }
        } else {
            Ok(CheckResult::new_user("There is no track currently playing"))
        }
    } else {
        Ok(CheckResult::new_user("You are not in a voice channel"))
    }
}
