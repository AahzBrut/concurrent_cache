use std::ops::Deref;
use std::sync::Arc;
use std::thread;
use std::time::Duration;


use concurrent_lru::sharded::LruCache;
use rand::Rng;
use rand::rngs::ThreadRng;

#[derive(Debug)]
struct CacheEntry {
    id: usize,
    data: Vec<Vec<String>>,
}

impl Drop for CacheEntry {
    fn drop(&mut self) {
        println!("Cube {} is evicted", self.id);
    }
}

struct CacheManager {
    cache: LruCache<usize, Arc<CacheEntry>>,
}

impl CacheManager {
    pub fn new(size: usize) -> Self {
        Self {
            cache: LruCache::new(size as u64),
        }
    }

    pub fn get_entry(&self, id: usize) -> Arc<CacheEntry> {
        self.cache.get_or_init(id, 1, |entry_id| { CacheManager::get_data(*entry_id) });
        while self.cache.get(id).is_none() {
            thread::sleep(Duration::from_millis(1));
        }
        self.cache.get(id).unwrap().value().clone()
    }

    fn get_data(id: usize) -> Arc<CacheEntry> {
        println!("Loading data for cube: {}", id);
        Arc::new(CacheEntry { id, data: vec![vec![format!("Data for cube with id: {}", id)]]})
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
            let entry_id = rng.gen_range(0..20);
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
