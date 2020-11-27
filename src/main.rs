mod commands;
mod db;
mod state;

use serenity::{
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    prelude::*,
};

use songbird::SerenityInit;

use sqlx::PgPool;

use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
};

use tracing_subscriber::{EnvFilter, FmtSubscriber};

use commands::{
    db_testing::*, help::*, join::*, leave::*, loop_command::*, mute::*, now_playing::*, pause::*,
    ping::*, play::*, queue::*, remove::*, restart::*, resume::*, skip::*, stop::*, volume::*,
};

use commands::perms::admin::*;

use state::*;

#[group]
#[commands(
    ping,
    join,
    leave,
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
    now_playing
)]
struct General;

#[group]
#[commands(
    get_author_perms,
    set_author_perms,
    get_perms_in_guild,
    delete_author,
    delete_current_guild
)]
struct Owner;

#[group]
#[commands(admin)]
struct Moderation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    let token = env::var("DISCORD_TOKEN")?;

    let http = Http::new_with_token(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("~"))
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&MODERATION_GROUP)
        .group(&OWNER_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .register_songbird()
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<SongMetadataContainer>(Arc::new(RwLock::new(HashMap::new())));
        data.insert::<PoolContainer>(pool);
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
