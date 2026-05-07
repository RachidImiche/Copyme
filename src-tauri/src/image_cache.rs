use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

const MAX_CACHE_ITEMS: usize = 8;

#[derive(Clone)]
pub struct CachedImage {
    pub width: usize,
    pub height: usize,
    pub bytes: Arc<Vec<u8>>,
}

struct ImageCache {
    order: VecDeque<String>,
    items: HashMap<String, CachedImage>,
}

impl ImageCache {
    fn new() -> Self {
        Self {
            order: VecDeque::new(),
            items: HashMap::new(),
        }
    }

    fn put(&mut self, hash: &str, width: usize, height: usize, bytes: &[u8]) {
        if self.items.contains_key(hash) {
            self.order.retain(|h| h != hash);
        }

        self.items.insert(
            hash.to_string(),
            CachedImage {
                width,
                height,
                bytes: Arc::new(bytes.to_vec()),
            },
        );
        self.order.push_back(hash.to_string());

        while self.order.len() > MAX_CACHE_ITEMS {
            if let Some(old) = self.order.pop_front() {
                self.items.remove(&old);
            }
        }
    }

    fn put_owned(&mut self, hash: &str, width: usize, height: usize, bytes: Vec<u8>) {
        if self.items.contains_key(hash) {
            self.order.retain(|h| h != hash);
        }

        self.items.insert(
            hash.to_string(),
            CachedImage {
                width,
                height,
                bytes: Arc::new(bytes),
            },
        );
        self.order.push_back(hash.to_string());

        while self.order.len() > MAX_CACHE_ITEMS {
            if let Some(old) = self.order.pop_front() {
                self.items.remove(&old);
            }
        }
    }

    fn get(&self, hash: &str) -> Option<CachedImage> {
        self.items.get(hash).cloned()
    }
}

static IMAGE_CACHE: OnceLock<Mutex<ImageCache>> = OnceLock::new();

fn cache() -> &'static Mutex<ImageCache> {
    IMAGE_CACHE.get_or_init(|| Mutex::new(ImageCache::new()))
}

pub fn put_image(hash: &str, width: usize, height: usize, bytes: &[u8]) {
    if let Ok(mut c) = cache().lock() {
        c.put(hash, width, height, bytes);
    }
}

pub fn put_image_owned(hash: &str, width: usize, height: usize, bytes: Vec<u8>) {
    if let Ok(mut c) = cache().lock() {
        c.put_owned(hash, width, height, bytes);
    }
}

pub fn get_image(hash: &str) -> Option<CachedImage> {
    cache().lock().ok().and_then(|c| c.get(hash))
}
