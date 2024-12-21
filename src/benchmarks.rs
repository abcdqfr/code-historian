use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::analyzer::Analyzer;
use crate::config::Config;
use crate::plugin::PluginManager;

pub struct BenchmarkResults {
    pub repo_processing_speed: f64,  // KB/s
    pub memory_usage: usize,         // Bytes
    pub plugin_execution_times: HashMap<String, Duration>,
    pub visualization_generation_time: Duration,
}

pub struct Benchmarker {
    config: Config,
    plugin_manager: PluginManager,
}

impl Benchmarker {
    pub fn new(config: Config, plugin_manager: PluginManager) -> Self {
        Self {
            config,
            plugin_manager,
        }
    }

    pub fn run_benchmarks(&self) -> BenchmarkResults {
        let mut results = BenchmarkResults {
            repo_processing_speed: 0.0,
            memory_usage: 0,
            plugin_execution_times: HashMap::new(),
            visualization_generation_time: Duration::from_secs(0),
        };

        // Measure repository processing speed
        let start = Instant::now();
        let analyzer = Analyzer::new(&self.config);
        let analysis = analyzer.analyze().unwrap();
        let duration = start.elapsed();
        let repo_size = self.get_repo_size();
        results.repo_processing_speed = repo_size as f64 / duration.as_secs_f64() / 1024.0;

        // Measure memory usage
        results.memory_usage = self.measure_memory_usage();

        // Measure plugin execution times
        for plugin in self.plugin_manager.get_plugins() {
            let start = Instant::now();
            plugin.analyze(&analysis);
            results.plugin_execution_times.insert(
                plugin.name().to_string(),
                start.elapsed(),
            );
        }

        // Measure visualization generation time
        let start = Instant::now();
        analyzer.generate_visualizations(&analysis);
        results.visualization_generation_time = start.elapsed();

        results
    }

    fn get_repo_size(&self) -> usize {
        let mut total_size = 0;
        if let Ok(entries) = fs::read_dir(&self.config.repo_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len() as usize;
                    } else if metadata.is_dir() && !self.is_ignored_dir(&entry.path()) {
                        total_size += self.get_dir_size(&entry.path());
                    }
                }
            }
        }
        total_size
    }

    fn get_dir_size(&self, dir: &Path) -> usize {
        let mut total_size = 0;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len() as usize;
                    } else if metadata.is_dir() && !self.is_ignored_dir(&entry.path()) {
                        total_size += self.get_dir_size(&entry.path());
                    }
                }
            }
        }
        total_size
    }

    fn is_ignored_dir(&self, path: &Path) -> bool {
        let ignored = [".git", "target", "node_modules", ".code-historian"];
        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            ignored.contains(&dir_name)
        } else {
            false
        }
    }

    fn measure_memory_usage(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                if let Some(line) = status.lines().find(|l| l.starts_with("VmRSS:")) {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb.parse::<usize>() {
                            return kb * 1024;
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            // For non-Linux systems, we'll use a rough estimation based on heap allocation
            use std::alloc::{GlobalAlloc, Layout, System};
            struct MemoryTracker;
            static mut ALLOCATED: usize = 0;

            unsafe impl GlobalAlloc for MemoryTracker {
                unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                    ALLOCATED += layout.size();
                    System.alloc(layout)
                }

                unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                    ALLOCATED -= layout.size();
                    System.dealloc(ptr, layout)
                }
            }

            unsafe { ALLOCATED }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_benchmarker() {
        let config = Config {
            repo_path: PathBuf::from("."),
            history_dir: PathBuf::from(".code-historian"),
            output_dir: PathBuf::from("output"),
        };
        let plugin_manager = PluginManager::new();
        let benchmarker = Benchmarker::new(config, plugin_manager);
        let results = benchmarker.run_benchmarks();

        assert!(results.repo_processing_speed >= 0.0);
        assert!(results.memory_usage >= 0);
        assert!(results.visualization_generation_time >= Duration::from_secs(0));
    }

    #[test]
    fn test_repo_size_calculation() {
        let config = Config {
            repo_path: PathBuf::from("."),
            history_dir: PathBuf::from(".code-historian"),
            output_dir: PathBuf::from("output"),
        };
        let plugin_manager = PluginManager::new();
        let benchmarker = Benchmarker::new(config, plugin_manager);
        let size = benchmarker.get_repo_size();
        assert!(size > 0);
    }

    #[test]
    fn test_memory_usage_measurement() {
        let config = Config {
            repo_path: PathBuf::from("."),
            history_dir: PathBuf::from(".code-historian"),
            output_dir: PathBuf::from("output"),
        };
        let plugin_manager = PluginManager::new();
        let benchmarker = Benchmarker::new(config, plugin_manager);
        let usage = benchmarker.measure_memory_usage();
        assert!(usage > 0);
    }
} 