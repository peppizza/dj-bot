use std::{collections::HashMap, sync::Arc};

use serenity::{model::id::UserId, prelude::*};
use songbird::Call;

pub type AuthorContainerLock = Arc<RwLock<HashMap<uuid::Uuid, UserId>>>;

pub async fn remove_entries_from_author_container(
    handler_lock: Arc<Mutex<Call>>,
    author_container_lock: AuthorContainerLock,
) {
    tracing::error!("good");

    let handler = handler_lock.lock().await;
    let queue = handler.queue();

    if !queue.is_empty() {
        let current_queue = queue.current_queue();
        let mut author_container = author_container_lock.write().await;

        for track in current_queue {
            author_container.remove(&track.uuid());
        }

        tracing::error!("{:?}", author_container);
    }

    queue.stop();
}
