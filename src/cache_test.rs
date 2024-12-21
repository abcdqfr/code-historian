use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use crate::cache::{CacheManager, CacheMetadata};
use crate::Result;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};
use std::collections::HashMap;

async fn setup_cache() -> (CacheManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let cache_manager = CacheManager::new(
        temp_dir.path().to_path_buf(),
        1024 * 1024 * 10, // 10MB
    ).unwrap();
    (cache_manager, temp_dir)
}

#[tokio::test]
async fn test_cache_hit_rate() -> Result<()> {
    let (mut cache_manager, _temp_dir) = setup_cache().await;
    let mut hits = 0;
    let mut misses = 0;
    let total_ops = 1000;

    // Populate cache with initial data
    for i in 0..500 {
        let data = vec![i as u8; 100];
        cache_manager.put(&format!("key{}", i), &data, true)?;
    }

    // Perform random access pattern
    let mut rng = thread_rng();
    for _ in 0..total_ops {
        let key = format!("key{}", rng.gen_range(0..1000));
        if cache_manager.get::<Vec<u8>>(&key)?.is_some() {
            hits += 1;
        } else {
            misses += 1;
        }
    }

    let hit_rate = (hits as f64) / (total_ops as f64);
    println!("Cache hit rate: {:.2}%", hit_rate * 100.0);
    assert!(hit_rate > 0.4, "Hit rate should be above 40%");
    Ok(())
}

#[tokio::test]
async fn test_cache_eviction_effectiveness() -> Result<()> {
    let (mut cache_manager, _temp_dir) = setup_cache().await;
    let mut access_counts = HashMap::new();

    // Add entries with varying access patterns
    for i in 0..100 {
        let data = vec![i as u8; 100];
        cache_manager.put(&format!("key{}", i), &data, true)?;
        
        // Simulate different access patterns
        let accesses = match i {
            0..=19 => 10,  // Hot data
            20..=49 => 5,  // Warm data
            _ => 1,        // Cold data
        };

        for _ in 0..accesses {
            cache_manager.get::<Vec<u8>>(&format!("key{}", i))?;
        }
        access_counts.insert(format!("key{}", i), accesses);
    }

    // Force eviction by adding more data
    for i in 100..200 {
        let data = vec![i as u8; 100];
        cache_manager.put(&format!("key{}", i), &data, true)?;
    }

    // Check that frequently accessed items are still in cache
    let mut retained_hot = 0;
    let mut retained_warm = 0;
    let mut retained_cold = 0;

    for i in 0..100 {
        let key = format!("key{}", i);
        if cache_manager.get::<Vec<u8>>(&key)?.is_some() {
            match i {
                0..=19 => retained_hot += 1,
                20..=49 => retained_warm += 1,
                _ => retained_cold += 1,
            }
        }
    }

    println!("Retained hot items: {}/20", retained_hot);
    println!("Retained warm items: {}/30", retained_warm);
    println!("Retained cold items: {}/50", retained_cold);

    assert!(retained_hot > retained_warm / 2, "Hot items should be retained more than warm items");
    assert!(retained_warm > retained_cold, "Warm items should be retained more than cold items");
    Ok(())
}

#[tokio::test]
async fn test_cache_compression_effectiveness() -> Result<()> {
    let (mut cache_manager, _temp_dir) = setup_cache().await;
    let test_data = vec![
        ("text", vec![b'a'; 1000]),  // Highly compressible
        ("binary", thread_rng().gen::<[u8; 1000]>().to_vec()),  // Less compressible
    ];

    let mut compression_ratios = Vec::new();

    for (name, data) in test_data {
        // Store with compression
        cache_manager.put(name, &data, true)?;
        
        // Get metadata to check compressed size
        let entry = cache_manager.get_metadata(name)?;
        let compression_ratio = data.len() as f64 / entry.size as f64;
        compression_ratios.push((name, compression_ratio));
        
        // Verify data integrity
        let retrieved: Vec<u8> = cache_manager.get(name)?.unwrap();
        assert_eq!(data, retrieved, "Data integrity check failed for {}", name);
    }

    for (name, ratio) in compression_ratios {
        println!("Compression ratio for {}: {:.2}x", name, ratio);
        if name == "text" {
            assert!(ratio > 2.0, "Text compression should achieve at least 2x compression");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_cache_concurrent_access() -> Result<()> {
    let (cache_manager, _temp_dir) = setup_cache().await;
    let cache = std::sync::Arc::new(tokio::sync::Mutex::new(cache_manager));
    let mut handles = vec![];

    // Spawn multiple tasks doing concurrent reads and writes
    for i in 0..10 {
        let cache = cache.clone();
        let handle = tokio::spawn(async move {
            let mut success_count = 0;
            let mut error_count = 0;

            for j in 0..100 {
                let cache = cache.lock().await;
                let key = format!("key{}-{}", i, j);
                let data = vec![j as u8; 100];

                match cache.put(&key, &data, true) {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            (success_count, error_count)
        });
        handles.push(handle);
    }

    let mut total_success = 0;
    let mut total_errors = 0;

    for handle in handles {
        let (success, errors) = handle.await.unwrap();
        total_success += success;
        total_errors += errors;
    }

    println!("Concurrent operations - Success: {}, Errors: {}", total_success, total_errors);
    assert!(total_success > 900, "Should have mostly successful operations");
    assert!(total_errors < 100, "Should have few errors");
    Ok(())
}

#[tokio::test]
async fn test_cache_memory_usage() -> Result<()> {
    let (mut cache_manager, _temp_dir) = setup_cache().await;
    let initial_memory = get_process_memory();

    // Add significant amount of data to cache
    for i in 0..1000 {
        let data = vec![i as u8; 1000];
        cache_manager.put(&format!("key{}", i), &data, true)?;
    }

    let peak_memory = get_process_memory();
    
    // Clear cache
    cache_manager.clear()?;
    
    let final_memory = get_process_memory();

    println!("Memory usage (MB) - Initial: {}, Peak: {}, Final: {}", 
        initial_memory as f64 / 1024.0 / 1024.0,
        peak_memory as f64 / 1024.0 / 1024.0,
        final_memory as f64 / 1024.0 / 1024.0
    );

    assert!(peak_memory > initial_memory, "Cache should use additional memory");
    assert!(final_memory < peak_memory, "Memory should be released after clear");
    Ok(())
}

#[tokio::test]
async fn test_cache_persistence() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    // Create and populate cache
    {
        let mut cache_manager = CacheManager::new(cache_dir.clone(), 1024 * 1024)?;
        for i in 0..100 {
            let data = vec![i as u8; 100];
            cache_manager.put(&format!("key{}", i), &data, true)?;
        }
    }

    // Simulate process restart by creating new cache manager
    let mut cache_manager = CacheManager::new(cache_dir, 1024 * 1024)?;
    
    let mut restored_count = 0;
    for i in 0..100 {
        if let Some(data) = cache_manager.get::<Vec<u8>>(&format!("key{}", i))? {
            assert_eq!(data, vec![i as u8; 100]);
            restored_count += 1;
        }
    }

    println!("Restored {} items after restart", restored_count);
    assert_eq!(restored_count, 100, "All items should be restored");
    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("cache_operations", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (mut cache_manager, _temp_dir) = setup_cache().await;
                let data = black_box(vec![1u8; 1000]);
                
                // Benchmark put operation
                cache_manager.put("bench_key", &data, true).unwrap();
                
                // Benchmark get operation
                let _: Option<Vec<u8>> = cache_manager.get("bench_key").unwrap();
                
                // Benchmark update operation
                cache_manager.put("bench_key", &data, true).unwrap();
                
                // Benchmark remove operation
                cache_manager.remove("bench_key").unwrap();
            });
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn get_process_memory() -> usize {
    // Note: This is a simplified version. In a real implementation,
    // you would use platform-specific APIs to get accurate memory usage.
    std::process::Command::new("ps")
        .args(&["--no-headers", "-o", "rss", &format!("{}", std::process::id())])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<usize>()
                .unwrap_or(0)
        })
        .unwrap_or(0)
} 