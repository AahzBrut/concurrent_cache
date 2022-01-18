use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::thread;
use std::thread::Thread;
use std::time::{Duration, Instant};
use parking_lot::{Mutex, RwLock};


use rand::Rng;
use rand::rngs::ThreadRng;

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
}

impl CacheManager {
    pub fn new(size: usize) -> Self {
        Self {
            max_size: size,
            cache: RwLock::new(HashMap::new()),
            last_access: RwLock::new(HashMap::new()),
            entity_locks: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_entry(&self, id: usize) -> Arc<CacheEntry> {
        let mut guard = self.cache.write();
        self.last_access.write().insert(id, Instant::now());
        let mut entry_mutex: Arc<Mutex<i8>>;
        if guard.contains_key(&id) {
            return guard.get(&id).unwrap().clone();
        } else {
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

            if !self.entity_locks.read().contains_key(&id) {
                self.entity_locks.write().insert(id, Arc::new(Mutex::new(0i8)));
            }
            let tmp_lock = self.entity_locks.read();
            entry_mutex = tmp_lock.get(&id).unwrap().clone();
        }
        drop(guard);
        let mut guard = entry_mutex.lock();
        *guard = 0;
        if self.cache.read().contains_key(&id) {
            return self.cache.read().get(&id).unwrap().clone();
        } else {
            self.cache.write().insert(id, Arc::new(CacheEntry { data: vec![vec![format!("Data for cube {}", id)]]}));
        }
        let entry = self.cache.read().get(&id).unwrap().clone();
        *guard = 1;
        drop(guard);
        entry
    }
}

fn main() {
    let cache_manager = Arc::new(CacheManager::new(10));
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
            println!("Data in thread number: {}, is: {:?}", i, cache_entry);
        });
        threads.push(thread);
    }
    for thread in threads {
        thread.join().unwrap();
    }
}
