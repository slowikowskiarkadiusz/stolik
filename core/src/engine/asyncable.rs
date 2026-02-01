use std::{
    sync::{
        OnceLock, RwLock,
        mpsc::{Receiver, Sender, channel},
    },
    u16,
};

use crate::engine::components::world::World;

pub type AsyncableId = u16;
pub type AsyncableFunction = Box<dyn FnMut(&mut World, f32) + Send + Sync + 'static>;
pub enum AsyncableType {
    Timeout,
    Interval,
}
pub enum AsyncableActionType {
    Add,
    Remove,
}

static QUEUE: OnceLock<Sender<EnqueuedAsyncableJob>> = OnceLock::new();
static TAKEN_IDS: OnceLock<RwLock<Vec<AsyncableId>>> = OnceLock::new();
fn get_taken_ids() -> &'static RwLock<Vec<AsyncableId>> {
    TAKEN_IDS.get_or_init(|| RwLock::new(Vec::new()))
}

pub struct EnqueuedAsyncableJob {
    pub id_to_affect: AsyncableId,
    pub action_type: AsyncableActionType,
    pub asyncable: Option<AsyncableInProgress>,
}

pub struct AsyncableInProgress {
    pub id: AsyncableId,
    pub function: AsyncableFunction,
    pub async_type: AsyncableType,
    pub ms: f32,
    pub timer: f32,
}

pub struct AsyncableStorage {
    asyncables_in_progress: Vec<AsyncableInProgress>,
    queue_receiver: Receiver<EnqueuedAsyncableJob>,
}

impl AsyncableStorage {
    pub fn new() -> Self {
        let (s, r) = channel();
        QUEUE.set(s).unwrap();
        Self {
            asyncables_in_progress: Vec::new(),
            queue_receiver: r,
        }
    }

    pub fn update(&mut self, world: &mut World, delta_time: f32) {
        while let Ok(job) = self.queue_receiver.try_recv() {
            match job.action_type {
                AsyncableActionType::Add => {
                    self.asyncables_in_progress.push(job.asyncable.unwrap());
                }
                AsyncableActionType::Remove => {
                    self.asyncables_in_progress.retain(|f| f.id != job.id_to_affect);
                }
            }
        }

        for asyncable in self.asyncables_in_progress.iter_mut() {
            asyncable.timer += delta_time;

            match asyncable.async_type {
                AsyncableType::Timeout => {
                    if asyncable.timer >= asyncable.ms {
                        (asyncable.function)(world, delta_time);
                        remove_asyncable(asyncable.id);
                    }
                }
                AsyncableType::Interval => {
                    if asyncable.timer >= asyncable.ms {
                        asyncable.timer = 0.0;
                        (asyncable.function)(world, delta_time);
                    }
                }
            }
        }
    }
}

pub fn add_asyncable(function: AsyncableFunction, ms: f32, asyncable_type: AsyncableType) -> AsyncableId {
    let lock = get_taken_ids().read().unwrap();
    let mut free_id = 0;
    for i in 0..=u16::MAX {
        // TODO optimize this shit
        free_id = i;
        if !lock.contains(&free_id) {
            break;
        }
    }

    let job = EnqueuedAsyncableJob {
        id_to_affect: free_id.clone(),
        action_type: AsyncableActionType::Add,
        asyncable: Some(AsyncableInProgress {
            id: free_id.clone(),
            function,
            async_type: asyncable_type,
            ms,
            timer: 0.0,
        }),
    };

    drop(lock);

    get_taken_ids().write().unwrap().push(free_id.clone());
    get_taken_ids().write().unwrap().sort();
    QUEUE.get().unwrap().send(job).unwrap();

    free_id
}

pub fn remove_asyncable(id: AsyncableId) {
    let job = EnqueuedAsyncableJob {
        id_to_affect: id.clone(),
        action_type: AsyncableActionType::Remove,
        asyncable: None,
    };

    get_taken_ids().write().unwrap().retain_mut(|f| f != &id);
    QUEUE.get().unwrap().send(job).unwrap();
}
