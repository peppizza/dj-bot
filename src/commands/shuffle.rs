use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use std::collections::VecDeque;

use rand::thread_rng;

trait LenAndSwap {
    fn len(&self) -> usize;
    fn swap(&mut self, i: usize, j: usize);
}

fn shuffle_queue<T, R>(values: &mut T, mut rng: R)
where
    T: LenAndSwap,
    R: rand::Rng,
{
    let mut i = values.len();
    while i >= 2 {
        i -= 1;
        values.swap(i, rng.gen_range(0, i + 1))
    }
}

impl<T> LenAndSwap for VecDeque<T> {
    fn len(&self) -> usize {
        self.len()
    }
    fn swap(&mut self, i: usize, j: usize) {
        self.swap(i, j)
    }
}

#[command]
#[only_in(guilds)]
async fn shuffle(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        if queue.is_empty() {
            msg.reply_mention(ctx, "The queue is currently empty")
                .await?;
            return Ok(());
        }

        queue.modify_queue(|queue| {
            let playing_track = queue.pop_front().unwrap();
            shuffle_queue(queue, thread_rng());
            queue.push_front(playing_track);
        });

        msg.channel_id.say(ctx, "Shuffled queue").await?;
    }

    Ok(())
}
