use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use crate::state::SongMetadataContainer;

#[command]
#[only_in(guilds)]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let index = args.single_quoted::<usize>()?;

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        if !queue.is_empty() {
            if index == 0 {
                {
                    let current = queue.current().unwrap();
                    let data = ctx.data.write().await;
                    let metadata_container_lock =
                        data.get::<SongMetadataContainer>().unwrap().clone();
                    let mut metadata_container = metadata_container_lock.write().await;

                    metadata_container.remove(&current.uuid());
                }

                queue.skip()?;

                msg.channel_id.say(ctx, "Skipped the song").await?;
            } else if index > queue.len() {
                msg.reply(ctx, format!("There is no song at index: {}", index))
                    .await?;
                return Ok(());
            } else {
                let title = {
                    let uuid = queue.dequeue(index).unwrap().uuid();
                    let data = ctx.data.write().await;
                    let metadata_container_lock =
                        data.get::<SongMetadataContainer>().unwrap().clone();
                    let mut metadata_container = metadata_container_lock.write().await;

                    metadata_container.remove(&uuid)
                }
                .unwrap()
                .title
                .unwrap();

                msg.channel_id
                    .say(ctx, format!("Removed song: `{}`", title))
                    .await?;
            }
        } else {
            msg.channel_id.say(ctx, "The queue is empty").await?;
        }
    }

    Ok(())
}
