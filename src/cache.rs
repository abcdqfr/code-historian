use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::{Result, HistorianError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cache {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub entries: HashMap<String, CacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub metadata: CacheMetadata,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub size: u64,
    pub hash: String,
    pub content_type: String,
    pub compression: bool,
}

pub struct CacheManager {
    cache_dir: PathBuf,
    max_size: u64,
    cache: Cache,
}

impl CacheManager {
    pub fn new(cache_dir: PathBuf, max_size: u64) -> Result<Self> {
        fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("cache.json");
        let cache = if cache_file.exists() {
            let data = fs::read_to_string(&cache_file)?;
            serde_json::from_str(&data)?
        } else {
            Cache {
                version: env!("CARGO_PKG_VERSION").to_string(),
                created_at: Utc::now(),
                last_accessed: Utc::now(),
                entries: HashMap::new(),
            }
        };

        Ok(Self {
            cache_dir,
            max_size,
            cache,
        })
    }

    pub fn get<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>> {
        if let Some(entry) = self.cache.entries.get_mut(key) {
            entry.last_accessed = Utc::now();
            entry.access_count += 1;
            self.save_cache()?;

            let data = if entry.metadata.compression {
                let compressed = fs::read(self.cache_dir.join(&entry.key))?;
                zstd::decode_all(&compressed[..])?
            } else {
                fs::read(self.cache_dir.join(&entry.key))?
            };

            Ok(Some(serde_json::from_slice(&data)?))
        } else {
            Ok(None)
        }
    }

    pub fn put<T: Serialize>(&mut self, key: &str, value: &T, compress: bool) -> Result<()> {
        let data = serde_json::to_vec(value)?;
        let hash = sha2::Sha256::digest(&data);
        let hash = format!("{:x}", hash);

        let (final_data, compression) = if compress {
            (zstd::encode_all(&data[..], 3)?, true)
        } else {
            (data, false)
        };

        let entry = CacheEntry {
            key: key.to_string(),
            data: final_data.clone(),
            metadata: CacheMetadata {
                size: final_data.len() as u64,
                hash,
                content_type: "application/json".to_string(),
                compression,
            },
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
        };

        // Check cache size and evict if necessary
        self.ensure_cache_size(entry.metadata.size)?;

        // Save data
        fs::write(self.cache_dir.join(key), &final_data)?;

        // Update cache
        self.cache.entries.insert(key.to_string(), entry);
        self.cache.last_accessed = Utc::now();
        self.save_cache()?;

        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> Result<()> {
        if let Some(_) = self.cache.entries.remove(key) {
            let path = self.cache_dir.join(key);
            if path.exists() {
                fs::remove_file(path)?;
            }
            self.save_cache()?;
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        for key in self.cache.entries.keys() {
            let path = self.cache_dir.join(key);
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        self.cache.entries.clear();
        self.save_cache()?;
        Ok(())
    }

    fn ensure_cache_size(&mut self, new_entry_size: u64) -> Result<()> {
        let mut current_size: u64 = self.cache.entries.values()
            .map(|e| e.metadata.size)
            .sum();

        if current_size + new_entry_size > self.max_size {
            // Sort entries by last accessed time
            let mut entries: Vec<_> = self.cache.entries.iter().collect();
            entries.sort_by_key(|(_, e)| e.last_accessed);

            // Remove entries until we have enough space
            while current_size + new_entry_size > self.max_size {
                if let Some((key, entry)) = entries.pop() {
                    current_size -= entry.metadata.size;
                    self.remove(key)?;
                } else {
                    break;
                }
            }
        }

        Ok(())
    }

    fn save_cache(&self) -> Result<()> {
        let cache_file = self.cache_dir.join("cache.json");
        let data = serde_json::to_string_pretty(&self.cache)?;
        fs::write(cache_file, data)?;
        Ok(())
    }

    pub fn get_metadata(&self, key: &str) -> Result<CacheMetadata> {
        if let Some(entry) = self.cache.entries.get(key) {
            Ok(entry.metadata.clone())
        } else {
            Err(HistorianError::InvalidArgument(format!("Cache entry not found: {}", key)))
        }
    }

    pub fn get_stats(&self) -> CacheStats {
        let total_size: u64 = self.cache.entries.values()
            .map(|e| e.metadata.size)
            .sum();

        let total_accesses: u64 = self.cache.entries.values()
            .map(|e| e.access_count)
            .sum();

        let avg_access_count = if self.cache.entries.is_empty() {
            0.0
        } else {
            total_accesses as f64 / self.cache.entries.len() as f64
        };

        CacheStats {
            total_entries: self.cache.entries.len(),
            total_size,
            used_percentage: (total_size as f64 / self.max_size as f64) * 100.0,
            avg_access_count,
            created_at: self.cache.created_at,
            last_accessed: self.cache.last_accessed,
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size: u64,
    pub used_percentage: f64,
    pub avg_access_count: f64,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache_manager = CacheManager::new(
            temp_dir.path().to_path_buf(),
            1024 * 1024, // 1MB
        ).unwrap();

        // Test put
        let data = vec![1, 2, 3, 4];
        cache_manager.put("test", &data, true).unwrap();

        // Test get
        let retrieved: Vec<u8> = cache_manager.get("test").unwrap().unwrap();
        assert_eq!(data, retrieved);

        // Test remove
        cache_manager.remove("test").unwrap();
        assert!(cache_manager.get::<Vec<u8>>("test").unwrap().is_none());

        // Test clear
        cache_manager.put("test1", &vec![1, 2, 3], true).unwrap();
        cache_manager.put("test2", &vec![4, 5, 6], true).unwrap();
        cache_manager.clear().unwrap();
        assert!(cache_manager.get::<Vec<u8>>("test1").unwrap().is_none());
        assert!(cache_manager.get::<Vec<u8>>("test2").unwrap().is_none());
    }

    #[test]
    fn test_cache_size_management() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache_manager = CacheManager::new(
            temp_dir.path().to_path_buf(),
            100, // Small cache size for testing
        ).unwrap();

        // Add entries until cache is full
        for i in 0..10 {
            let data = vec![i; 10]; // 10 bytes each
            cache_manager.put(&format!("test{}", i), &data, false).unwrap();
        }

        // Add one more entry, should evict oldest
        let data = vec![10; 10];
        cache_manager.put("test10", &data, false).unwrap();

        // First entry should be evicted
        assert!(cache_manager.get::<Vec<u8>>("test0").unwrap().is_none());
        
        // Latest entry should exist
        assert!(cache_manager.get::<Vec<u8>>("test10").unwrap().is_some());
    }
} 