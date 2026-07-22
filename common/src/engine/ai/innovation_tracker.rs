extern crate alloc;
use alloc::{collections::BTreeMap, vec::Vec};
use embassy_sync::once_lock::OnceLock;
use spin::Mutex;

use crate::engine::ai::neat_genome::{AtomicId, Id};
use core::sync::atomic::Ordering;

static CONNECTION_INNOVATIONS: OnceLock<Mutex<BTreeMap<(Id, Id), Id>>> = OnceLock::new();
static NEXT_INNOVATION_ID: AtomicId = AtomicId::new(0);
static NODE_SPLIT_INNOVATIONS: OnceLock<Mutex<BTreeMap<Id, Id>>> = OnceLock::new();
static NEXT_NODE_ID: AtomicId = AtomicId::new(0);

pub fn get_innovation_id(node_in: Id, node_out: Id) -> Id {
    let mut lock = CONNECTION_INNOVATIONS
        .get_or_init(|| Mutex::new(BTreeMap::<(Id, Id), Id>::new()))
        .lock();

    if let Some(id) = lock.get(&(node_in, node_out)) {
        *id
    } else {
        let id = get_next_innovation_id();
        lock.insert((node_in, node_out), id);
        id
    }
}

pub fn get_node_split_innovation(split_connection: Id) -> Id {
    let mut lock = NODE_SPLIT_INNOVATIONS
        .get_or_init(|| Mutex::new(BTreeMap::<Id, Id>::new()))
        .lock();

    if let Some(id) = lock.get(&split_connection) {
        *id
    } else {
        let id = get_next_node_id();
        lock.insert(split_connection, id);
        id
    }
}

fn get_next_innovation_id() -> Id {
    NEXT_INNOVATION_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn get_next_node_id() -> Id {
    NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn reset_generation_cache() {
    CONNECTION_INNOVATIONS
        .get_or_init(|| Mutex::new(BTreeMap::<(Id, Id), Id>::new()))
        .lock()
        .clear();

    NODE_SPLIT_INNOVATIONS
        .get_or_init(|| Mutex::new(BTreeMap::<Id, Id>::new()))
        .lock()
        .clear();
}
