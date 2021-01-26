use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::MessageBuilder,
};

#[command]
#[owners_only]
#[help_available(false)]
async fn sys_info(ctx: &Context, msg: &Message) -> CommandResult {
    let cpu_load = sys_info::loadavg()?;
    let mem_use = sys_info::mem_info()?;

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title("sys-info");
                e.field(
                    "CPU load average",
                    format!("{:.2}%", cpu_load.one * 10.0),
                    false,
                );
                e.field(
                    "Memory Usage",
                    format!(
                        "{:.2} MB Free out of {:.2} MB",
                        mem_use.free as f32 / 1000.0,
                        mem_use.total as f32 / 1000.0,
                    ),
                    false,
                );

                e
            })
        })
        .await?;

    Ok(())
}

#[command]
#[owners_only]
#[help_available(false)]
async fn in_voice_channel(ctx: &Context, msg: &Message) -> CommandResult {
    let guilds = ctx.cache.guilds().await;
    let mut res = MessageBuilder::new();
    for guild_id in guilds {
        let guild = match guild_id.to_guild_cached(&ctx.cache).await {
            Some(guild) => guild,
            None => continue,
        };

        let bot_channel_id = guild.voice_states.get(&ctx.cache.current_user_id().await);

        if let Some(bot_channel_id) = bot_channel_id {
            res.push_line(format!("{:?}", bot_channel_id.guild_id));
        }
    }

    if res.0.is_empty() {
        msg.channel_id.say(ctx, "Not in any vc's").await?;
    } else {
        msg.channel_id.say(ctx, res.build()).await?;
    }

    Ok(())
}

#[command]
#[owners_only]
#[help_available(false)]
async fn guild_count(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(ctx, ctx.cache.guild_count().await)
        .await?;

    Ok(())
}
