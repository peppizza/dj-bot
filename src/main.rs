mod checks;
mod commands;
mod consts;
mod data;
mod db;
mod dj_only_store;
mod events;
mod lyrics_api;
mod playlists;
mod queue;
mod voice_events;

use bb8_redis::{bb8, RedisConnectionManager};
use db::get_guild_prefix;
use serenity::{
    client::bridge::gateway::GatewayIntents,
    framework::standard::Reason,
    framework::{
        standard::{
            buckets::LimitedFor,
            macros::{group, hook},
            DispatchError,
        },
        StandardFramework,
    },
    http::Http,
    model::channel::Message,
    prelude::*,
    FutureExt,
};

use songbird::SerenityInit;

use sqlx::PgPool;
use tracing::{info, warn};
use tracing_log::env_logger;

use std::{collections::HashSet, env, time::Duration};

use tracing_subscriber::{EnvFilter, FmtSubscriber};

use commands::{
    db_testing::*, dj_only::*, help::*, info::*, join::*, loop_command::*, lyrics::*, mute::*,
    now_playing::*, pause::*, perms::*, ping::*, play::*, prefix::*, queue::*, remove::*,
    restart::*, resume::*, shuffle::*, skip::*, stop::*, volume::*,
};

use data::*;
use events::Handler;
use queue::QueueMap;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[group]
#[commands(
    ping,
    join,
    mute,
    play,
    skip,
    stop,
    loop_command,
    remove,
    volume,
    pause,
    resume,
    restart,
    queue,
    now_playing,
    shuffle,
    donate,
    lyrics,
    bot_info
)]
struct General;

#[group]
#[commands(
    get_author_perms,
    set_author_perms,
    get_perms_in_guild,
    delete_author,
    delete_current_guild,
    insert_all_guilds,
    sys_info,
    in_voice_channel,
    guild_count
)]
struct Owner;

#[group]
#[commands(perms, dj_only, prefix)]
struct Moderation;

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::CheckFailed(_, reason) => match reason {
            Reason::Log(log) => info!("{:?}", log),
            Reason::User(reason) => {
                let _ = msg.reply_ping(ctx, reason).await;
            }
            _ => {}
        },
        DispatchError::Ratelimited(duration) => {
            if duration.as_secs() == 0 {
                let _ = msg
                    .channel_id
                    .say(
                        ctx,
                        format!("Try this again in {}ms.", duration.as_millis()),
                    )
                    .await;
            } else {
                let _ = msg
                    .channel_id
                    .say(
                        ctx,
                        format!("Try this again in {} seconds.", duration.as_secs()),
                    )
                    .await;
            }
        }
        e => warn!("{:?}", e),
    }
}

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    env_logger::init();

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    let manager = RedisConnectionManager::new(env::var("REDIS_URL")?)?;

    let redis_pool = bb8::Pool::builder()
        .connection_timeout(Duration::from_secs(5))
        .build(manager)
        .await?;

    let token = env::var("DISCORD_TOKEN")?;

    let http = Http::new_with_token(&token);

    let owners = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            if let Some(team) = info.team {
                for member in team.members {
                    owners.insert(member.user.id);
                }
            }

            owners
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.owners(owners)
                .prefix("")
                .dynamic_prefix(|ctx, msg| {
                    async move {
                        let guild_id = msg.guild_id.unwrap();
                        let data = ctx.data.read().await;
                        let prefix_cache = data.get::<PrefixCache>().unwrap().clone();
                        let pool = data.get::<PoolContainer>().unwrap();
                        let prefix =
                            match get_guild_prefix(pool, prefix_cache, guild_id.into()).await {
                                Ok(prefix) => prefix,
                                Err(e) => {
                                    tracing::error!("Problem getting prefix!!! {:?}", e);
                                    return None;
                                }
                            };

                        Some(prefix)
                    }
                    .boxed()
                })
                .case_insensitivity(true)
        })
        .on_dispatch_error(dispatch_error)
        .after(|ctx, msg, cmd_name, error| {
            async move {
                if let Err(e) = error {
                    warn!("Error with command {}, {:?}", cmd_name, e);
                    let _ = msg.channel_id.say(ctx, format!("Command returned an error, {:?}, please report this on the support server https://discord.gg/5YytF9fPHr", e)).await;
                }
            }
            .boxed()
        })
        .bucket("global", |b| b.delay(3).limit_for(LimitedFor::User))
        .await
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&MODERATION_GROUP)
        .group(&OWNER_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler::new())
        .register_songbird()
        .intents(
            GatewayIntents::GUILDS
                | GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILD_VOICE_STATES
                | GatewayIntents::GUILD_MEMBERS,
        )
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<PoolContainer>(pool);
        data.insert::<ReqwestClientContainer>(Default::default());
        data.insert::<DjOnlyContainer>(redis_pool);
        data.insert::<QueueMap>(Default::default());
        data.insert::<PrefixCache>(Default::default());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    client.start_autosharded().await?;

    Ok(())
}
