use std::{collections::VecDeque, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use dashmap::DashMap;
use parking_lot::Mutex;
use serenity::{
    async_trait,
    client::Context,
    http::Http,
    model::id::{ChannelId, GuildId},
    prelude::{Mutex as AsyncMutex, TypeMapKey},
};
use songbird::{
    input::{Input, Restartable},
    tracks::{create_player_with_uuid, TrackHandle},
    Call, Event, EventContext, EventHandler, TrackEvent,
};
use tracing::{info, warn};
use uuid::Uuid;

use crate::voice_events::TrackStartNotifier;

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
    current_track: Arc<Mutex<Option<TrackHandle>>>,
    next_track: Mutex<Option<TrackHandle>>,
}
struct PlayNextTrack {
    remote_lock: Arc<Mutex<QueueCore>>,
    driver: Arc<AsyncMutex<Call>>,
    chan_id: ChannelId,
    http: Arc<Http>,
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

            if let Some(next_track) = inner.next_track.lock().as_ref() {
                let _ = next_track.play();
                let mut current_track = inner.current_track.lock();
                *current_track = Some(next_track.clone())
            } else {
                let mut current_track = inner.current_track.lock();
                *current_track = None;
            }

            info!("Queued track ended: {:?}.", ctx);
            info!("{} tracks remain.", inner.tracks.len());
        }

        loop {
            let next_track = {
                let inner = self.remote_lock.lock();

                inner.tracks.get(1).cloned()
            };

            if let Some(next_track) = next_track {
                let next_track_uuid = next_track.uuid;
                let input = match get_input_from_queued_track(next_track).await {
                    Ok(i) => i,
                    Err(e) => {
                        warn!("Could not play track {:?}", e);
                        let mut inner = self.remote_lock.lock();
                        inner.tracks.remove(1);
                        continue;
                    }
                };

                let (track, handle) = create_player_with_uuid(input, next_track_uuid);
                let _ = handle.add_event(
                    Event::Track(TrackEvent::End),
                    Self {
                        driver: self.driver.clone(),
                        remote_lock: self.remote_lock.clone(),
                        chan_id: self.chan_id,
                        http: self.http.clone(),
                    },
                );
                let _ = handle.add_event(
                    Event::Delayed(Duration::from_millis(5)),
                    TrackStartNotifier {
                        chan_id: self.chan_id,
                        http: self.http.clone(),
                    },
                );
                let _ = handle.pause();
                let mut handler = self.driver.lock().await;
                handler.play(track);
                let inner = self.remote_lock.lock();
                let mut next_track = inner.next_track.lock();
                *next_track = Some(handle);
                break;
            } else {
                break;
            }
        }
        None
    }
}

async fn get_input_from_queued_track(track: QueuedTrack) -> Result<Input> {
    match Restartable::ytdl_search(&track.name, true).await {
        Ok(r) => Ok(r.into()),
        Err(e) => Err(anyhow!("{:?}", e)),
    }
}

impl Queue {
    pub async fn add(
        &self,
        input: QueuedTrack,
        driver: Arc<AsyncMutex<Call>>,
        chan_id: ChannelId,
        http: Arc<Http>,
    ) -> anyhow::Result<()> {
        let (search, input_uuid) = {
            let mut inner = self.inner.lock();
            let search = input.name.clone();
            let input_uuid = input.uuid;
            inner.tracks.push_back(input);
            (search, input_uuid)
        };
        if self.len() == 1 {
            let input = match Restartable::ytdl_search(&search, true).await {
                Ok(input) => input,
                Err(e) => return Err(anyhow!("{:?}", e)),
            };
            let (track, handle) = create_player_with_uuid(Input::from(input), input_uuid);
            handle.add_event(
                Event::Track(TrackEvent::End),
                PlayNextTrack {
                    driver: driver.clone(),
                    remote_lock: self.inner.clone(),
                    chan_id,
                    http,
                },
            )?;
            let mut handler = driver.lock().await;
            handler.play(track);
            let inner = self.inner.lock();
            let mut current_track = inner.current_track.lock();
            *current_track = Some(handle);
        } else if self.len() == 2 {
            let input = match Restartable::ytdl_search(&search, true).await {
                Ok(i) => i,
                Err(e) => return Err(anyhow!("{:?}", e)),
            };
            let (track, handle) = create_player_with_uuid(Input::from(input), input_uuid);
            handle.add_event(
                Event::Track(TrackEvent::End),
                PlayNextTrack {
                    driver: driver.clone(),
                    remote_lock: self.inner.clone(),
                    chan_id,
                    http: http.clone(),
                },
            )?;
            handle.add_event(
                Event::Track(TrackEvent::End),
                TrackStartNotifier { chan_id, http },
            )?;
            handle.pause()?;
            let mut handler = driver.lock().await;
            handler.play(track);
            let inner = self.inner.lock();
            let mut next_track = inner.next_track.lock();
            *next_track = Some(handle);
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

        if let Some(handle) = inner.current_track.lock().as_ref() {
            let _ = handle.stop();
        }

        if let Some(handle) = inner.next_track.lock().as_ref() {
            let _ = handle.stop();
        }

        inner.tracks.clear();
    }

    pub fn skip(&mut self) -> anyhow::Result<()> {
        let inner = self.inner.lock();

        if let Some(handle) = inner.current_track.lock().as_ref() {
            handle.stop()?;
        }
        Ok(())
    }

    pub fn current(&self) -> Arc<Mutex<Option<TrackHandle>>> {
        let inner = self.inner.lock();

        inner.current_track.clone()
    }

    pub fn current_queue(&self) -> Vec<QueuedTrack> {
        let inner = self.inner.lock();

        inner.tracks.iter().cloned().collect()
    }

    pub fn dequeue(&self, index: usize) -> Option<QueuedTrack> {
        if index == 0 {
            let inner = self.inner.lock();
            let current_track = inner.current_track.lock();
            if let Some(handle) = current_track.as_ref() {
                let _ = handle.stop();
            }
        } else if index == 1 {
            let inner = self.inner.lock();
            let next_track = inner.next_track.lock();
            if let Some(handle) = next_track.as_ref() {
                let _ = handle.stop();
            }
        }

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
    type Value = Arc<DashMap<GuildId, Queue>>;
}

pub async fn get_queue_from_ctx_and_guild_id(ctx: &Context, guild_id: GuildId) -> Queue {
    let data = ctx.data.read().await;
    let queue_container = data.get::<QueueMap>().unwrap().clone();
    let queue = queue_container.get(&guild_id).unwrap().clone();

    queue
}
