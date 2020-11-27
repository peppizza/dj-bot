pub mod admin;
pub mod blacklist;
pub mod dj;
mod util {
    use serenity::{
        framework::standard::{macros::check, Args, CheckResult, CommandOptions},
        model::prelude::*,
        prelude::*,
        utils::parse_mention,
    };

    use crate::{
        db::{get_user_perms, UserPerm},
        state::PoolContainer,
    };

    pub const INSUFFICIENT_PERMISSIONS_MESSAGE: &str =
        "You have insufficient permissions to run this command";

    pub async fn args_to_user(
        ctx: &Context,
        msg: &Message,
        mut args: Args,
    ) -> anyhow::Result<Option<User>> {
        let mentioned_user = match args.single_quoted::<String>() {
            Ok(user) => user,
            Err(_) => {
                msg.reply(
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
                msg.reply(ctx, "Not a valid mention").await?;
                return Ok(None);
            }
        };

        let user = match UserId(mentioned_user).to_user(ctx).await {
            Ok(user) => user,
            Err(_) => {
                msg.reply(ctx, "Not a valid user").await?;
                return Ok(None);
            }
        };

        Ok(Some(user))
    }

    #[check]
    #[name = "Perms"]
    pub async fn perms_check(
        ctx: &Context,
        msg: &Message,
        _: &mut Args,
        _: &CommandOptions,
    ) -> CheckResult {
        let guild = msg.guild(ctx).await.unwrap();
        let perms = guild.member_permissions(ctx, msg.author.id).await.unwrap();

        if perms.administrator() {
            CheckResult::Success
        } else {
            let data = ctx.data.read().await;
            let pool = data.get::<PoolContainer>().unwrap();

            if let Some(perm_level) = get_user_perms(pool, guild.id.into(), msg.author.id.into())
                .await
                .unwrap()
            {
                if let UserPerm::Admin = perm_level {
                    CheckResult::Success
                } else {
                    CheckResult::new_user(INSUFFICIENT_PERMISSIONS_MESSAGE)
                }
            } else {
                CheckResult::new_user(INSUFFICIENT_PERMISSIONS_MESSAGE)
            }
        }
    }
}
