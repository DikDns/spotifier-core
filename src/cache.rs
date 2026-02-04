use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

/// Trait for cache storage backends.
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Retrieve a value from the cache.
    async fn get(&self, key: &str) -> Option<String>;
    /// Store a value in the cache with a Time-To-Live (TTL).
    async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), String>;
    /// Remove a value from the cache.
    async fn delete(&self, key: &str) -> Result<(), String>;
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    data: String,
    expires_at: u64,
}

/// Simple file-based cache backend.
pub struct FileCache {
    cache_dir: PathBuf,
}

impl FileCache {
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
