//! # slow-rs
//!
//! A comprehensive system slowness diagnostic tool for Linux workstations.
//!
//! ## Overview
//!
//! `slow-rs` helps identify the root cause of mysterious system slowdowns by
//! continuously monitoring and benchmarking various system metrics. It's
//! particularly useful when you're not sure whether a slowdown is caused by:
//!
//! - Disk I/O problems (failing drive, high latency)
//! - Memory pressure (swapping, OOM conditions)
//! - CPU issues (thermal throttling, VM steal time)
//! - General resource exhaustion
//!
//! ## Features
//!
//! - **Active Benchmarks**: Measures I/O throughput, memory allocation speed,
//!   and CPU compute performance at regular intervals
//! - **Passive Monitoring**: Collects detailed stats from `/proc` including
//!   CPU time breakdown, disk I/O, network traffic, and memory details
//! - **Pressure Stall Information (PSI)**: Reports Linux PSI metrics to show
//!   when tasks are waiting for CPU, memory, or I/O
//! - **Temperature Monitoring**: Tracks CPU and system temperatures
//! - **TUI Dashboard**: Real-time terminal UI with charts
//! - **CSV Logging**: All metrics logged for later analysis
//!
//! ## Usage
//!
//! ```bash
//! # Run with TUI (default)
//! slow-rs
//!
//! # Headless mode for logging only
//! slow-rs --headless
//!
//! # Custom interval and skip I/O benchmark
//! slow-rs -i 10 --skip-io-bench
//! ```
//!
//! ## Module Organization
//!
//! - [`config`]: CLI argument parsing and configuration
//! - [`metrics`]: Data structures for collected metrics
//! - [`collectors`]: Functions to read system stats from `/proc`
//! - [`benchmarks`]: Active performance tests
//! - [`app`]: Main application state and coordination
//! - [`ui`]: Terminal user interface

mod app;
mod benchmarks;
mod collectors;
mod config;
mod metrics;
mod ui;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;

use app::App;
use config::Config;

fn main() -> std::io::Result<()> {
    let config = Config::parse();
    let app = App::new(config.clone())?;

    // Create test file if needed
    app.ensure_test_file()?;

    // Setup Ctrl+C / SIGTERM handler
    let running = Arc::new(AtomicBool::new(true));
    setup_signal_handler(running.clone());

    let interval = Duration::from_secs(config.interval);

    if config.headless {
        ui::run_headless(app, running, interval)?;
    } else {
        ui::run(app, running, interval)?;
    }

    Ok(())
}

/// Set up signal handlers for graceful shutdown.
fn setup_signal_handler(running: Arc<AtomicBool>) {
    // Store the running flag in a static for the signal handler
    *RUNNING_FLAG.lock().unwrap() = Some(running);

    unsafe {
        libc::signal(
            libc::SIGINT,
            signal_handler as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            signal_handler as *const () as libc::sighandler_t,
        );
    }
}

/// Global storage for the running flag (needed for signal handler).
static RUNNING_FLAG: std::sync::Mutex<Option<Arc<AtomicBool>>> = std::sync::Mutex::new(None);

/// Signal handler that sets the running flag to false.
extern "C" fn signal_handler(_: i32) {
    if let Some(ref running) = *RUNNING_FLAG.lock().unwrap() {
        running.store(false, Ordering::Relaxed);
    }
}
