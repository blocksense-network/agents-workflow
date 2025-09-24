//! Stress testing utilities for FUSE integration tests

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::task;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "fuse-stress-test")]
#[command(about = "Stress testing utilities for FUSE integration tests")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Run concurrent file operations stress test
    Concurrent {
        /// Number of concurrent operations
        #[arg(short, long, default_value = "50")]
        concurrency: usize,

        /// Number of operations per thread
        #[arg(short, long, default_value = "100")]
        operations: usize,

        /// Test mount point
        mount_point: PathBuf,
    },
    /// Run memory pressure stress test
    Memory {
        /// Number of files to create
        #[arg(short, long, default_value = "10000")]
        file_count: usize,

        /// File size in bytes
        #[arg(short, long, default_value = "1024")]
        file_size: usize,

        /// Test mount point
        mount_point: PathBuf,
    },
    /// Run directory stress test
    Directory {
        /// Number of directories to create
        #[arg(short, long, default_value = "100")]
        dir_count: usize,

        /// Files per directory
        #[arg(short, long, default_value = "100")]
        files_per_dir: usize,

        /// Test mount point
        mount_point: PathBuf,
    },
    /// Run performance benchmark
    Benchmark {
        /// Benchmark duration in seconds
        #[arg(short, long, default_value = "60")]
        duration_secs: u64,

        /// Test mount point
        mount_point: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    match args.command {
        Commands::Concurrent { concurrency, operations, mount_point } => {
            run_concurrent_stress_test(concurrency, operations, mount_point).await?;
        }
        Commands::Memory { file_count, file_size, mount_point } => {
            run_memory_stress_test(file_count, file_size, mount_point).await?;
        }
        Commands::Directory { dir_count, files_per_dir, mount_point } => {
            run_directory_stress_test(dir_count, files_per_dir, mount_point).await?;
        }
        Commands::Benchmark { duration_secs, mount_point } => {
            run_performance_benchmark(duration_secs, mount_point).await?;
        }
    }

    Ok(())
}

async fn run_concurrent_stress_test(
    concurrency: usize,
    operations: usize,
    mount_point: PathBuf,
) -> Result<()> {
    info!("Running concurrent stress test: {} concurrent ops, {} ops each", concurrency, operations);

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = vec![];

    let start_time = Instant::now();

    for thread_id in 0..concurrency {
        let mount_point = mount_point.clone();
        let semaphore = semaphore.clone();

        let handle = task::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            for op_id in 0..operations {
                let filename = format!("stress_thread_{}_op_{}.txt", thread_id, op_id);
                let filepath = mount_point.join(&filename);
                let content = format!("Thread {} operation {}", thread_id, op_id);

                // Write file
                if let Err(e) = fs::write(&filepath, &content) {
                    warn!("Failed to write {}: {}", filepath.display(), e);
                    continue;
                }

                // Read and verify
                match fs::read_to_string(&filepath) {
                    Ok(read_content) if read_content == content => {
                        // Success, clean up
                        let _ = fs::remove_file(&filepath);
                    }
                    Ok(read_content) => {
                        warn!("Content mismatch for {}: expected '{}', got '{}'",
                              filepath.display(), content, read_content);
                    }
                    Err(e) => {
                        warn!("Failed to read {}: {}", filepath.display(), e);
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await?;
    }

    let total_time = start_time.elapsed();
    let total_ops = concurrency * operations;
    let ops_per_sec = total_ops as f64 / total_time.as_secs_f64();

    info!("✅ Concurrent stress test completed:");
    info!("  Total operations: {}", total_ops);
    info!("  Total time: {:?}", total_time);
    info!("  Operations/sec: {:.2}", ops_per_sec);

    Ok(())
}

async fn run_memory_stress_test(
    file_count: usize,
    file_size: usize,
    mount_point: PathBuf,
) -> Result<()> {
    info!("Running memory stress test: {} files, {} bytes each", file_count, file_size);

    let content = "x".repeat(file_size);
    let start_time = Instant::now();

    // Create files
    for i in 0..file_count {
        let filename = format!("memory_stress_{}.bin", i);
        let filepath = mount_point.join(&filename);

        if let Err(e) = fs::write(&filepath, &content) {
            warn!("Failed to create file {}: {}", filepath.display(), e);
        }

        if i % 1000 == 0 {
            info!("Created {} files...", i);
        }
    }

    let create_time = start_time.elapsed();
    info!("File creation completed in {:?}", create_time);

    // Verify files
    let verify_start = Instant::now();
    let mut verified = 0;
    let mut errors = 0;

    for i in 0..file_count {
        let filename = format!("memory_stress_{}.bin", i);
        let filepath = mount_point.join(&filename);

        match fs::read(&filepath) {
            Ok(data) if data == content.as_bytes() => {
                verified += 1;
            }
            Ok(data) => {
                warn!("File {} has wrong content (size: {})", filepath.display(), data.len());
                errors += 1;
            }
            Err(e) => {
                warn!("Failed to read {}: {}", filepath.display(), e);
                errors += 1;
            }
        }
    }

    let verify_time = verify_start.elapsed();

    // Cleanup
    let cleanup_start = Instant::now();
    for i in 0..file_count {
        let filename = format!("memory_stress_{}.bin", i);
        let filepath = mount_point.join(&filename);
        let _ = fs::remove_file(&filepath);
    }
    let cleanup_time = cleanup_start.elapsed();

    info!("✅ Memory stress test completed:");
    info!("  Files created: {}", file_count);
    info!("  Create time: {:?}", create_time);
    info!("  Files verified: {}", verified);
    info!("  Errors: {}", errors);
    info!("  Verify time: {:?}", verify_time);
    info!("  Cleanup time: {:?}", cleanup_time);

    Ok(())
}

async fn run_directory_stress_test(
    dir_count: usize,
    files_per_dir: usize,
    mount_point: PathBuf,
) -> Result<()> {
    info!("Running directory stress test: {} dirs, {} files each", dir_count, files_per_dir);

    let start_time = Instant::now();

    // Create directories and files
    for dir_id in 0..dir_count {
        let dirname = format!("stress_dir_{}", dir_id);
        let dirpath = mount_point.join(&dirname);

        if let Err(e) = fs::create_dir(&dirpath) {
            warn!("Failed to create directory {}: {}", dirpath.display(), e);
            continue;
        }

        // Create files in directory
        for file_id in 0..files_per_dir {
            let filename = format!("file_{}.txt", file_id);
            let filepath = dirpath.join(&filename);
            let content = format!("Directory {} file {}", dir_id, file_id);

            if let Err(e) = fs::write(&filepath, &content) {
                warn!("Failed to create file {}: {}", filepath.display(), e);
            }
        }

        if dir_id % 10 == 0 {
            info!("Created {} directories...", dir_id);
        }
    }

    let create_time = start_time.elapsed();

    // Verify directories
    let verify_start = Instant::now();
    let mut verified_dirs = 0;
    let mut verified_files = 0;
    let mut errors = 0;

    for dir_id in 0..dir_count {
        let dirname = format!("stress_dir_{}", dir_id);
        let dirpath = mount_point.join(&dirname);

        match fs::read_dir(&dirpath) {
            Ok(entries) => {
                let file_count = entries.count();
                if file_count == files_per_dir {
                    verified_dirs += 1;
                    verified_files += file_count;
                } else {
                    warn!("Directory {} has {} files, expected {}", dirpath.display(), file_count, files_per_dir);
                    errors += 1;
                }
            }
            Err(e) => {
                warn!("Failed to read directory {}: {}", dirpath.display(), e);
                errors += 1;
            }
        }
    }

    let verify_time = verify_start.elapsed();

    // Cleanup
    let cleanup_start = Instant::now();
    for dir_id in 0..dir_count {
        let dirname = format!("stress_dir_{}", dir_id);
        let dirpath = mount_point.join(&dirname);

        // Remove files first
        if let Ok(entries) = fs::read_dir(&dirpath) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }

        // Remove directory
        let _ = fs::remove_dir(&dirpath);
    }
    let cleanup_time = cleanup_start.elapsed();

    info!("✅ Directory stress test completed:");
    info!("  Directories created: {}", dir_count);
    info!("  Files per directory: {}", files_per_dir);
    info!("  Create time: {:?}", create_time);
    info!("  Directories verified: {}", verified_dirs);
    info!("  Files verified: {}", verified_files);
    info!("  Errors: {}", errors);
    info!("  Verify time: {:?}", verify_time);
    info!("  Cleanup time: {:?}", cleanup_time);

    Ok(())
}

async fn run_performance_benchmark(
    duration_secs: u64,
    mount_point: PathBuf,
) -> Result<()> {
    info!("Running performance benchmark for {} seconds", duration_secs);

    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(duration_secs);

    let mut operations = 0;
    let mut file_counter = 0;

    while Instant::now() < end_time {
        // Create file
        let filename = format!("bench_{}.txt", file_counter);
        let filepath = mount_point.join(&filename);
        let content = format!("benchmark content {}", file_counter);

        if fs::write(&filepath, &content).is_ok() {
            // Read file
            if fs::read_to_string(&filepath).is_ok() {
                // Delete file
                let _ = fs::remove_file(&filepath);
                operations += 3; // create + read + delete
            } else {
                operations += 1; // just create
            }
        }

        file_counter += 1;
    }

    let elapsed = start_time.elapsed();
    let ops_per_sec = operations as f64 / elapsed.as_secs_f64();

    info!("✅ Performance benchmark completed:");
    info!("  Duration: {:?}", elapsed);
    info!("  Operations: {}", operations);
    info!("  Operations/sec: {:.2}", ops_per_sec);

    Ok(())
}
