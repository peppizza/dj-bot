mod commands;
mod state;

use serenity::{
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    prelude::*,
};

use songbird::SerenityInit;

use std::{collections::HashSet, env};

use tracing_subscriber::{EnvFilter, FmtSubscriber};

use commands::{
    help::*, join::*, leave::*, loop_command::*, mute::*, ping::*, play::*, remove::*, skip::*,
    stop::*, volume::*,
};

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
    volume
)]
struct General;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

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
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .register_songbird()
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
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
