pub mod admin;
pub mod dj;
pub mod user;
mod util {
    use serenity::{
        framework::standard::Args, model::prelude::*, prelude::*, utils::parse_mention,
    };

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
}
