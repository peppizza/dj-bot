use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use serenity::{
    async_trait,
    client::Context,
    model::id::GuildId,
    prelude::{Mutex as AsyncMutex, RwLock, TypeMapKey},
};
use songbird::{
    create_player,
    input::{Input, Restartable},
    tracks::TrackHandle,
    Call, Event, EventContext, EventHandler, TrackEvent,
};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct QueuedTrack {
    pub name: String,
    pub uuid: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct Queue {
    inner: Arc<Mutex<QueueCore>>,
}

#[derive(Debug, Default)]
pub struct QueueCore {
    tracks: VecDeque<QueuedTrack>,
    current_track: Option<TrackHandle>,
}

struct PlayNextTrack {
    remote_lock: Arc<Mutex<QueueCore>>,
    driver: Arc<AsyncMutex<Call>>,
}

#[async_trait]
impl EventHandler for PlayNextTrack {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        {
            let mut inner = self.remote_lock.lock();

            let front_ended = match ctx {
                EventContext::Track(ts) => {
                    let queue_uuid = inner.tracks.front().map(|input| input.uuid);
                    let ended_uuid = ts.first().map(|handle| handle.1.uuid());

                    queue_uuid.is_some() && queue_uuid == ended_uuid
                }
                _ => false,
            };

            if !front_ended {
                return None;
            }

            inner.tracks.pop_front();

            info!("Queued track ended: {:?}.", ctx);
            info!("{} tracks remain.", inner.tracks.len());
        }

        let mut should_pop = false;
        loop {
            let next_track = {
                let mut inner = self.remote_lock.lock();
                if should_pop {
                    inner.tracks.pop_front();
                }
                inner.tracks.front().cloned()
            };

            if let Some(next_track) = next_track {
                let next_track_uuid = next_track.uuid;
                let input = match get_input_from_queued_track(next_track).await {
                    Ok(i) => i,
                    Err(e) => {
                        warn!("Could not play track {:?}", e);
                        should_pop = true;
                        continue;
                    }
                };

                let (mut track, mut handle) = create_player(input);
                let _ = handle.add_event(
                    Event::Track(TrackEvent::End),
                    Self {
                        driver: self.driver.clone(),
                        remote_lock: self.remote_lock.clone(),
                    },
                );
                track.set_uuid(next_track_uuid);
                handle.set_uuid(next_track_uuid);
                let mut handler = self.driver.lock().await;
                handler.play(track);
                let mut inner = self.remote_lock.lock();
                inner.current_track = Some(handle);
                break;
            } else {
                break;
            }
        }

        None
    }
}

async fn get_input_from_queued_track(track: QueuedTrack) -> Result<Input> {
    match Restartable::ytdl_search(&track.name).await {
        Ok(r) => Ok(Input::from(r)),
        Err(e) => Err(anyhow!("{:?}", e)),
    }
}

impl Queue {
    pub async fn add(
        &self,
        input: QueuedTrack,
        driver: Arc<AsyncMutex<Call>>,
    ) -> anyhow::Result<()> {
        let (search, input_uuid) = {
            let mut inner = self.inner.lock();
            let search = input.name.clone();
            let input_uuid = input.uuid;
            inner.tracks.push_back(input);
            (search, input_uuid)
        };
        if self.len() == 1 {
            let input = match Restartable::ytdl_search(&search).await {
                Ok(input) => input,
                Err(e) => return Err(anyhow!("{:?}", e)),
            };
            let (mut track, mut handle) = create_player(Input::from(input));
            handle.add_event(
                Event::Track(TrackEvent::End),
                PlayNextTrack {
                    driver: driver.clone(),
                    remote_lock: self.inner.clone(),
                },
            )?;
            track.set_uuid(input_uuid);
            handle.set_uuid(input_uuid);
            let mut handler = driver.lock().await;
            handler.play(track);
            let mut inner = self.inner.lock();
            inner.current_track = Some(handle);
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();

        inner.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        let inner = self.inner.lock();

        inner.tracks.len()
    }

    pub fn stop(&self) {
        let mut inner = self.inner.lock();

        if let Some(handle) = &inner.current_track {
            let _ = handle.stop();
        }

        inner.tracks.clear();
    }

    pub fn skip(&mut self) -> anyhow::Result<()> {
        let inner = self.inner.lock();

        if let Some(handle) = &inner.current_track {
            handle.stop()?;
        }
        Ok(())
    }

    pub fn current(&self) -> Option<TrackHandle> {
        let inner = self.inner.lock();

        inner.current_track.clone()
    }

    pub fn current_queue(&self) -> Vec<QueuedTrack> {
        let inner = self.inner.lock();

        inner.tracks.iter().cloned().collect()
    }

    pub fn dequeue(&self, index: usize) -> Option<QueuedTrack> {
        self.modify_queue(|vq| vq.remove(index))
    }

    pub fn modify_queue<F, O>(&self, func: F) -> O
    where
        F: FnOnce(&mut VecDeque<QueuedTrack>) -> O,
    {
        let mut inner = self.inner.lock();

        func(&mut inner.tracks)
    }
}

pub struct QueueMap;

impl TypeMapKey for QueueMap {
    type Value = Arc<RwLock<HashMap<GuildId, Queue>>>;
}

pub async fn get_queue_from_ctx_and_guild_id(ctx: &Context, guild_id: GuildId) -> Queue {
    let data = ctx.data.read().await;
    let queue_container_lock = data.get::<QueueMap>().unwrap().clone();
    let queue_container = queue_container_lock.read().await;
    let queue = queue_container.get(&guild_id).unwrap().clone();

    queue
}
