use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use anyhow::anyhow;
use serenity::{
    async_trait,
    model::id::GuildId,
    prelude::{Mutex, RwLock, TypeMapKey},
};
use songbird::{
    create_player,
    input::{Input, Metadata, Restartable},
    tracks::{TrackHandle, TrackResult},
    Call, Driver, Event, EventContext, EventHandler, TrackEvent,
};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct QueuedTrack {
    metadata: Metadata,
    uuid: Uuid,
}

#[derive(Debug, Clone)]
pub struct Queue {
    inner: Arc<Mutex<VecDeque<QueuedTrack>>>,
    current_track: Arc<Mutex<Option<TrackHandle>>>,
}

struct PlayNextTrack {
    remote_lock: Arc<Mutex<VecDeque<QueuedTrack>>>,
    driver: Arc<Mutex<Call>>,
    current_track: Arc<Mutex<Option<TrackHandle>>>,
}

#[async_trait]
impl EventHandler for PlayNextTrack {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let mut inner = self.remote_lock.lock().await;

        let front_ended = match ctx {
            EventContext::Track(ts) => {
                let queue_uuid = inner.front().map(|input| input.uuid);
                let ended_uuid = ts.first().map(|handle| handle.1.uuid());

                queue_uuid.is_some() && queue_uuid == ended_uuid
            }
            _ => false,
        };

        if !front_ended {
            return None;
        }

        inner.pop_front();

        info!("Queued track ended: {:?}.", ctx);
        info!("{} tracks remain.", inner.len());

        while !inner.is_empty() {
            if let Some(new) = inner.front() {
                let url = new.metadata.clone().source_url.unwrap();
                let input = match Restartable::ytdl(url).await {
                    Ok(i) => Input::from(i),
                    Err(e) => {
                        warn!("Track in queue couldn't be played...");
                        inner.pop_front();
                        continue;
                    }
                };
                let (track, handle) = create_player(input);
                let mut driver = self.driver.lock().await;
                driver.play(track);
                let mut current_track = self.current_track.lock().await;
                *current_track = Some(handle);
            }
        }

        None
    }
}

impl Queue {
    pub async fn add(&self, input: QueuedTrack, driver: Arc<Mutex<Call>>) -> anyhow::Result<()> {
        let metadata = input.metadata.clone();
        {
            let mut inner = self.inner.lock().await;
            inner.push_front(input);
        }
        if self.is_empty().await {
            let url = metadata.source_url.clone();
            let input = match Restartable::ytdl(url.unwrap()).await {
                Ok(input) => input,
                Err(e) => return Err(anyhow!("{:?}", e)),
            };
            let (track, handle) = create_player(Input::from(input));
            handle.add_event(
                Event::Track(TrackEvent::End),
                PlayNextTrack {
                    current_track: self.current_track.clone(),
                    driver: driver.clone(),
                    remote_lock: self.inner.clone(),
                },
            );
            let mut handler = driver.lock().await;
            handler.play(track);
            let mut current_track = self.current_track.lock().await;
            *current_track = Some(handle);
        }
        Ok(())
    }

    pub async fn is_empty(&self) -> bool {
        let inner = self.inner.lock().await;

        inner.is_empty()
    }

    pub async fn len(&self) -> usize {
        let inner = self.inner.lock().await;

        inner.len()
    }

    pub async fn pause(&self) -> TrackResult<()> {
        let current_track = self.current_track.lock().await;
        if let Some(handle) = &*current_track {
            handle.pause()
        } else {
            Ok(())
        }
    }

    pub async fn resume(&self) -> TrackResult<()> {
        let current_track = self.current_track.lock().await;
        if let Some(handle) = &*current_track {
            handle.play()
        } else {
            Ok(())
        }
    }

    pub async fn stop(&self) {
        let current_track = self.current_track.lock().await;
        if let Some(handle) = &*current_track {
            let _ = handle.stop();
        }

        let mut inner = self.inner.lock().await;

        inner.clear();
    }

    pub async fn skip(&mut self, driver: &mut Driver) -> anyhow::Result<()> {
        let mut current_track = self.current_track.lock().await;
        if let Some(handle) = &*current_track {
            handle.stop()?;
            let mut inner = self.inner.lock().await;
            if let Some(next_track) = inner.pop_front() {
                let url = next_track.metadata.source_url;
                let input = match Restartable::ytdl(url.unwrap()).await {
                    Ok(i) => i,
                    Err(e) => return Err(anyhow!("{:?}", e)),
                };
                let (track, handle) = create_player(Input::from(input));
                driver.play(track);
                *current_track = Some(handle);
                Ok(())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn current(&self) -> Arc<Mutex<Option<TrackHandle>>> {
        self.current_track.clone()
    }

    pub async fn current_queue(&self) -> Vec<QueuedTrack> {
        let inner = self.inner.lock().await;

        inner.iter().map(|input| input.clone()).collect()
    }

    pub async fn dequeue(&self, index: usize) -> Option<QueuedTrack> {
        self.modify_queue(|vq| vq.remove(index)).await
    }

    pub async fn modify_queue<F, O>(&self, func: F) -> O
    where
        F: FnOnce(&mut VecDeque<QueuedTrack>) -> O,
    {
        let mut inner = self.inner.lock().await;
        func(&mut inner)
    }
}

impl Default for Queue {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
            current_track: Arc::new(Mutex::new(None)),
        }
    }
}

pub struct QueueMap;

impl TypeMapKey for QueueMap {
    type Value = Arc<RwLock<HashMap<GuildId, Queue>>>;
}
