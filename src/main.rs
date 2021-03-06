mod sync;

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use parking_lot::{Mutex, RawRwLock, RwLock};
use parking_lot::lock_api::RwLockWriteGuard;
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::sync::Semaphore;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<Vec<String>>,
}

impl Drop for CacheEntry {
    fn drop(&mut self) {
        println!("Cube: {:?} dropped", self);
    }
}

struct CacheManager {
    max_size: usize,
    cache: RwLock<HashMap<usize, Arc<CacheEntry>>>,
    last_access: RwLock<HashMap<usize, Instant>>,
    entity_locks: RwLock<HashMap<usize, Arc<Mutex<i8>>>>,
    semaphore: Semaphore,
}

impl CacheManager {
    pub fn new(size: usize, load_mdp: usize) -> Self {
        Self {
            max_size: size,
            cache: RwLock::new(HashMap::new()),
            last_access: RwLock::new(HashMap::new()),
            entity_locks: RwLock::new(HashMap::new()),
            semaphore: Semaphore::new(load_mdp),
        }
    }

    pub fn get_entry(&self, id: usize) -> Arc<CacheEntry> {
        let mdp_guard = self.semaphore.acquire();
        let mut guard = self.cache.write();
        self.last_access.write().insert(id, Instant::now());
        let entry_mutex = if guard.contains_key(&id) {
            return guard.get(&id).unwrap().clone();
        } else {
            self.evict_oldest_entries(&mut guard);
            self.get_or_init_entry_mutex(id)
        };
        drop(guard);
        let entry = self.get_or_load_entry(id, &entry_mutex);
        drop(mdp_guard);
        entry
    }

    fn get_or_init_entry_mutex(&self, id: usize) -> Arc<Mutex<i8>> {
        if let Some(entry_lock) = self.entity_locks.read().get(&id) {
            return entry_lock.clone();
        }
        let entry_lock = Arc::new(Mutex::new(0i8));
        self.entity_locks.write().insert(id, entry_lock.clone());
        entry_lock
    }

    fn get_or_load_entry(&self, id: usize, entry_mutex: &Arc<Mutex<i8>>) -> Arc<CacheEntry> {
        let guard = entry_mutex.lock();
        if let Some(entry) = self.cache.read().get(&id) {
            return entry.clone();
        }
        thread::sleep(Duration::from_secs(1));
        let entry = Arc::new(CacheEntry { data: vec![vec![format!("Data for cube {}", id)]] });
        self.cache.write().insert(id, entry.clone());
        drop(guard);
        entry
    }

    fn evict_oldest_entries(&self, guard: &mut RwLockWriteGuard<RawRwLock, HashMap<usize, Arc<CacheEntry>>>) {
        while guard.len() >= self.max_size {
            let mut earliest_instant = Instant::now();
            let mut earliest_id = usize::MAX;
            for item in self.last_access.read().iter() {
                if earliest_instant > *item.1 {
                    earliest_id = *item.0;
                    earliest_instant = *item.1;
                }
            }
            guard.remove(&earliest_id);
            self.last_access.write().remove(&earliest_id);
            self.entity_locks.write().remove(&earliest_id);
        }
    }
}

fn main() {
    let cache_manager = Arc::new(CacheManager::new(10, 5));
    let mut threads = Vec::new();

    for i in 0..100 {
        let manager = cache_manager.clone();
        let thread = thread::spawn(move || {
            //println!("Thread {} started.", i);
            let mut rng = ThreadRng::default();
            let duration = rng.gen_range(10..=500);
            thread::sleep(Duration::from_millis(duration));
            let entry_id = rng.gen_range(0..30);
            //println!("Thread {} requested entry {}", i, entry_id);
            let cache_entry = manager.deref().get_entry(entry_id);
            println!("Data in thread number: {} ({}), is: {:?}", i, entry_id, cache_entry.data);
        });
        threads.push(thread);
    }
    for thread in threads {
        thread.join().unwrap();
    }
}
