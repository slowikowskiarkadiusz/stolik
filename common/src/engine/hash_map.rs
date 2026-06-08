extern crate alloc;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::ops::{Index, IndexMut};

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    len: usize,
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }
    pub fn with_capacity(capacity: usize) -> Self {
        let mut buckets = Vec::new();
        for _ in 0..capacity {
            buckets.push(Vec::new());
        }
        Self { buckets, len: 0 }
    }
    fn hash<Q: Hash + ?Sized>(&self, key: &Q) -> usize {
        let mut hasher = FnvHasher::default();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.buckets.len()
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let idx = self.hash(&key);
        let bucket = &mut self.buckets[idx];
        for (k, v) in bucket.iter_mut() {
            if k == &key {
                return Some(core::mem::replace(v, value));
            }
        }
        bucket.push((key, value));
        self.len += 1;
        None
    }
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let idx = self.hash(key);
        self.buckets[idx].iter().find(|(k, _)| k.borrow() == key).map(|(_, v)| v)
    }
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let idx = self.hash(key);
        self.buckets[idx].iter_mut().find(|(k, _)| k.borrow() == key).map(|(_, v)| v)
    }
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let idx = self.hash(key);
        let bucket = &mut self.buckets[idx];
        if let Some(pos) = bucket.iter().position(|(k, _)| k.borrow() == key) {
            self.len -= 1;
            Some(bucket.swap_remove(pos).1)
        } else {
            None
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        self.len = 0;
    }
    pub fn from<const N: usize>(arr: [(K, V); N]) -> Self {
        let mut map = HashMap::with_capacity(N);
        for (k, v) in arr {
            map.insert(k, v);
        }
        map
    }
    pub fn iter(&self) -> HashMapIter<'_, K, V> {
        HashMapIter {
            buckets: &self.buckets,
            bucket_idx: 0,
            item_idx: 0,
        }
    }
    pub fn iter_mut(&mut self) -> HashMapIterMut<'_, K, V> {
        HashMapIterMut {
            buckets: self.buckets.iter_mut().collect(),
            bucket_idx: 0,
            item_idx: 0,
        }
    }
}

// --- Immutable iterator ---

pub struct HashMapIter<'a, K, V> {
    buckets: &'a Vec<Vec<(K, V)>>,
    bucket_idx: usize,
    item_idx: usize,
}

impl<'a, K, V> Iterator for HashMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.bucket_idx >= self.buckets.len() {
                return None;
            }
            let bucket = &self.buckets[self.bucket_idx];
            if self.item_idx < bucket.len() {
                let (k, v) = &bucket[self.item_idx];
                self.item_idx += 1;
                return Some((k, v));
            }
            self.bucket_idx += 1;
            self.item_idx = 0;
        }
    }
}

impl<'a, K: Hash + Eq, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = HashMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// --- Mutable iterator ---

pub struct HashMapIterMut<'a, K, V> {
    buckets: Vec<&'a mut Vec<(K, V)>>,
    bucket_idx: usize,
    item_idx: usize,
}

impl<'a, K, V> Iterator for HashMapIterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.bucket_idx >= self.buckets.len() {
                return None;
            }
            let bucket = &mut self.buckets[self.bucket_idx];
            if self.item_idx < bucket.len() {
                let (k, v) = &mut bucket[self.item_idx];
                self.item_idx += 1;
                let k = unsafe { &*(k as *const K) };
                let v = unsafe { &mut *(v as *mut V) };
                return Some((k, v));
            }
            self.bucket_idx += 1;
            self.item_idx = 0;
        }
    }
}

impl<'a, K: Hash + Eq, V> IntoIterator for &'a mut HashMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = HashMapIterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// --- Owned iterator ---

pub struct HashMapIntoIter<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    bucket_idx: usize,
}

impl<K, V> Iterator for HashMapIntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.bucket_idx >= self.buckets.len() {
                return None;
            }
            let bucket = &mut self.buckets[self.bucket_idx];
            if !bucket.is_empty() {
                return Some(bucket.remove(0));
            }
            self.bucket_idx += 1;
        }
    }
}

impl<K: Hash + Eq, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = HashMapIntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        HashMapIntoIter {
            buckets: self.buckets,
            bucket_idx: 0,
        }
    }
}

// --- Index ---

impl<K, V> Index<&K> for HashMap<K, V>
where
    K: Hash + Eq,
{
    type Output = V;
    fn index(&self, key: &K) -> &Self::Output {
        self.get(key).expect("no entry found for key")
    }
}

impl<K, V> IndexMut<&K> for HashMap<K, V>
where
    K: Hash + Eq,
{
    fn index_mut(&mut self, key: &K) -> &mut Self::Output {
        self.get_mut(key).expect("no entry found for key")
    }
}

impl<K, V> Default for HashMap<K, V>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

// --- FNV Hasher ---

struct FnvHasher(u64);

impl Default for FnvHasher {
    fn default() -> Self {
        Self(0xcbf29ce484222325)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= *byte as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}
