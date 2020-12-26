use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use parking_lot::Mutex;
use serenity::{
    async_trait,
    model::id::GuildId,
    prelude::{RwLock, TypeMapKey},
};
use songbird::{
    create_player,
    input::Input,
    tracks::{TrackHandle, TrackResult},
    Driver, Event, EventContext, EventHandler,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct InputWithUuid {
    input: Input,
    uuid: Uuid,
}

#[derive(Debug)]
pub struct Queue {
    inner: Arc<Mutex<VecDeque<InputWithUuid>>>,
    playing: Option<TrackHandle>,
}

struct PlayNextTrack {
    remote_lock: Arc<Mutex<VecDeque<InputWithUuid>>>,
}

#[async_trait]
impl EventHandler for PlayNextTrack {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let mut inner = self.remote_lock.lock();

        let front_ended = match ctx {
            EventContext::Track(ts) {
                let queue_uuid = inner.front().map(|input| input.uuid);
                let ended_uuid = ts.first().map(|handle| handle.1.uuid());

                queue_uuid.is_some() && queue_uuid == ended_uuid
            },
            _ => false
        };

        if !front_ended {
            return None;
        }

        inner.pop_front();
    }
}

impl Queue {
    pub fn add(&mut self, input: InputWithUuid, driver: &mut Driver) -> TrackResult<()> {
        {
            let mut inner = self.inner.lock();
            inner.push_front(input);
        }
        if self.is_empty() {
            let (track, handle) = create_player(input.input);
            driver.play(track);
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();

        inner.is_empty()
    }

    pub fn len(&self) -> usize {
        let inner = self.inner.lock();

        inner.len()
    }

    pub fn pause(&self) -> TrackResult<()> {
        if let Some(handle) = self.playing {
            handle.pause()
        } else {
            Ok(())
        }
    }

    pub fn resume(&self) -> TrackResult<()> {
        if let Some(handle) = self.playing {
            handle.play()
        } else {
            Ok(())
        }
    }

    pub fn stop(&self) {
        if let Some(handle) = self.playing {
            let _ = handle.stop();
        }

        let mut inner = self.inner.lock();

        inner.clear();
    }

    pub async fn skip(&mut self, driver: &mut Driver) -> TrackResult<()> {
        if let Some(handle) = &self.playing {
            handle.stop()?;
            let mut inner = self.inner.lock();
            if let Some(next_track) = inner.pop_front() {
                let (track, handle) = create_player(next_track.input);
                driver.play(track);
                self.playing = Some(handle);
                Ok(())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn current(&self) -> Option<TrackHandle> {
        self.playing
    }

    pub fn current_queue(&self) -> Vec<InputWithUuid> {
        let inner = self.inner.lock();

        inner.iter().map(|input| *input).collect()
    }

    pub fn dequeue(&self, index: usize) -> Option<InputWithUuid> {
        self.modify_queue(|vq| vq.remove(index))
    }

    pub fn modify_queue<F, O>(&self, func: F) -> O
    where
        F: FnOnce(&mut VecDeque<InputWithUuid>) -> O,
    {
        let mut inner = self.inner.lock();
        func(&mut inner)
    }
}

impl Default for Queue {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
            playing: None,
        }
    }
}

pub struct QueueMap;

impl TypeMapKey for QueueMap {
    type Value = Arc<RwLock<HashMap<GuildId, Queue>>>;
}
