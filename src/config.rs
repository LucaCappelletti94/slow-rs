//! Command-line configuration for slow-rs.
//!
//! This module defines all CLI arguments using `clap` for parsing.
//! The configuration controls measurement intervals, output files,
//! benchmark parameters, and display mode.

use clap::Parser;

/// System slowness diagnostic monitor.
///
/// slow-rs continuously monitors system performance metrics and runs
/// benchmarks to help diagnose mysterious system slowdowns. It can
/// identify issues related to:
///
/// - Disk I/O problems (failing drives, high latency)
/// - Memory pressure (swapping, OOM conditions)
/// - CPU issues (thermal throttling, steal time in VMs)
/// - General resource exhaustion
///
/// # Examples
///
/// ```bash
/// # Run with TUI interface (default)
/// slow-rs
///
/// # Run in headless mode with 10-second intervals
/// slow-rs --headless -i 10
///
/// # Skip I/O benchmark if disk is suspected
/// slow-rs --skip-io-bench
/// ```
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Diagnose system slowdowns by monitoring performance metrics")]
pub struct Config {
    /// Interval in seconds between measurements.
    ///
    /// Lower values give more granular data but increase system load
    /// from the benchmarks. Recommended: 5-30 seconds.
    #[arg(short, long, default_value_t = 5)]
    pub interval: u64,

    /// Path to CSV log file.
    ///
    /// All collected metrics are appended to this file in CSV format.
    /// The file is created if it doesn't exist, and new data is appended
    /// if it does (headers are only written once).
    #[arg(short = 'c', long, default_value = "metrics.csv")]
    pub csv_file: String,

    /// Path to the test file for I/O benchmarks.
    ///
    /// This file is used to measure disk read/write speeds. It will be
    /// created automatically if it doesn't exist. Use a path on the
    /// disk you want to test.
    #[arg(short, long, default_value = "/tmp/slowtest.bin")]
    pub test_file: String,

    /// Size of test file in MB.
    ///
    /// Larger files give more accurate throughput measurements but
    /// take longer to read/write. 256MB is a good balance.
    #[arg(short, long, default_value_t = 256)]
    pub file_size_mb: usize,

    /// Number of data points to keep in memory for plotting.
    ///
    /// This controls how much history is shown in the TUI charts.
    /// At 5-second intervals, 120 points = 10 minutes of history.
    #[arg(long, default_value_t = 120)]
    pub history_size: usize,

    /// Run in headless mode (no TUI, just logging).
    ///
    /// Useful for running over SSH without terminal capabilities,
    /// or when you just want to log data without the UI.
    #[arg(long)]
    pub headless: bool,

    /// Skip I/O benchmark (useful if disk is already suspected).
    ///
    /// If you suspect the disk is failing or very slow, skipping
    /// the I/O benchmark prevents the tool from making things worse.
    /// System I/O stats from /proc are still collected.
    #[arg(long)]
    pub skip_io_bench: bool,
}
