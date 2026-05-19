use core::u16;

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::Receiver;
use spin::RwLock;

use crate::engine::components::world::World;

pub type AsyncableId = u16;
pub type AsyncableFunction = Box<dyn FnMut(&mut World, f32) + Send + Sync + 'static>;
pub enum AsyncableType {
    Timeout,
    Interval,
}
enum AsyncableActionType {
    Add,
    Remove,
}

const MAX_ASYNCABLES: usize = 32;

/// (ids array, count)
static TAKEN_IDS: RwLock<([AsyncableId; MAX_ASYNCABLES], usize)> =
    RwLock::new(([0; MAX_ASYNCABLES], 0));

static QUEUE: Channel<CriticalSectionRawMutex, EnqueuedAsyncableJob, 32> = Channel::new();

struct EnqueuedAsyncableJob {
    pub id_to_affect: AsyncableId,
    pub action_type: AsyncableActionType,
    pub asyncable: Option<AsyncableInProgress>,
}

struct AsyncableInProgress {
    pub id: AsyncableId,
    pub function: AsyncableFunction,
    pub async_type: AsyncableType,
    pub seconds: f32,
    pub timer: f32,
}

pub struct AsyncableStorage {
    asyncables_in_progress: Vec<AsyncableInProgress>,
    queue_receiver: Receiver<'static, CriticalSectionRawMutex, EnqueuedAsyncableJob, 32>,
}

impl AsyncableStorage {
    pub fn new() -> Self {
        Self {
            asyncables_in_progress: Vec::new(),
            queue_receiver: QUEUE.receiver(),
        }
    }

    pub fn update(&mut self, world: &mut World, delta_time: f32) {
        while let Ok(job) = self.queue_receiver.try_receive() {
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
                    if asyncable.timer >= asyncable.seconds {
                        (asyncable.function)(world, delta_time);
                        remove_asyncable(asyncable.id);
                    }
                }
                AsyncableType::Interval => {
                    if asyncable.timer >= asyncable.seconds {
                        asyncable.timer = 0.0;
                        (asyncable.function)(world, delta_time);
                    }
                }
            }
        }
    }
}

pub fn add_asyncable(function: AsyncableFunction, ms: f32, asyncable_type: AsyncableType) -> AsyncableId {
    let mut lock = TAKEN_IDS.write();
    let (ids, count) = &mut *lock;

    // Find first ID not already taken
    let mut free_id: AsyncableId = 0;
    'outer: loop {
        for i in 0..*count {
            if ids[i] == free_id {
                free_id = free_id.wrapping_add(1);
                continue 'outer;
            }
        }
        break;
    }

    if *count < MAX_ASYNCABLES {
        ids[*count] = free_id;
        *count += 1;
    }
    drop(lock);

    let job = EnqueuedAsyncableJob {
        id_to_affect: free_id,
        action_type: AsyncableActionType::Add,
        asyncable: Some(AsyncableInProgress {
            id: free_id,
            function,
            async_type: asyncable_type,
            seconds: ms,
            timer: 0.0,
        }),
    };

    QUEUE.sender().try_send(job).ok();
    free_id
}

pub fn remove_asyncable(id: AsyncableId) {
    let mut lock = TAKEN_IDS.write();
    let (ids, count) = &mut *lock;
    if let Some(pos) = ids[..*count].iter().position(|&x| x == id) {
        *count -= 1;
        ids[pos] = ids[*count];
    }
    drop(lock);

    let job = EnqueuedAsyncableJob {
        id_to_affect: id,
        action_type: AsyncableActionType::Remove,
        asyncable: None,
    };

    QUEUE.sender().try_send(job).ok();
}
