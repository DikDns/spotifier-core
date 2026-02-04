use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

/// A trait for defining custom cache storage engines.
///
/// This allows the scraper to persist data (like course lists or session cookies)
/// using different backends such as the local filesystem, memory, or external
/// stores like Redis.
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Retrieves a value from the cache by its key.
    ///
    /// Returns `Some(String)` if the key exists and is not expired, otherwise `None`.
    async fn get(&self, key: &str) -> Option<String>;

    /// Stores a value in the cache with a specific Time-To-Live (TTL).
    ///
    /// # Arguments
    /// * `key` - The unique identifier for the data.
    /// * `value` - The string content to store.
    /// * `ttl_secs` - Duration in seconds before the data expires.
    async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), String>;

    /// Removes a value from the cache.
    async fn delete(&self, key: &str) -> Result<(), String>;
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    data: String,
    expires_at: u64,
}

/// A default cache implementation that stores data as JSON files on the local filesystem.
///
/// It ensures data integrity through atomic writes (writing to a temporary file before renaming).
pub struct FileCache {
    cache_dir: PathBuf,
}

impl FileCache {
    /// Creates a new `FileCache` using the specified directory.
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
        }
    }

    fn get_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key))
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

#[async_trait]
impl CacheBackend for FileCache {
    async fn get(&self, key: &str) -> Option<String> {
        let path = self.get_path(key);
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).await.ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;

        if entry.expires_at < Self::now_secs() {
            let _ = fs::remove_file(&path).await;
            return None;
        }

        Some(entry.data)
    }

    async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), String> {
        if !self.cache_dir.exists() {
            fs::create_dir_all(&self.cache_dir)
                .await
                .map_err(|e| e.to_string())?;
        }

        let path = self.get_path(key);
        let tmp_path = path.with_extension("tmp");

        let entry = CacheEntry {
            data: value.to_string(),
            expires_at: Self::now_secs() + ttl_secs,
        };

        let json = serde_json::to_string(&entry).map_err(|e| e.to_string())?;

        // Atomic write: write to tmp then rename
        fs::write(&tmp_path, json)
            .await
            .map_err(|e| e.to_string())?;
        fs::rename(&tmp_path, &path)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), String> {
        let path = self.get_path(key);
        if path.exists() {
            fs::remove_file(path).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
