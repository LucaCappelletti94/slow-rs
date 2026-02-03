# slow-rs

A comprehensive system slowness diagnostic tool for Linux workstations. Designed to help identify the root cause of mysterious system slowdowns by continuously monitoring and benchmarking various system metrics.

## Features

### Live Benchmarks (run every interval)

- **I/O Read Speed**: Reads a test file and measures throughput (MB/s)
- **I/O Write Speed**: Writes to disk and measures throughput (MB/s)  
- **SHA256 Compute**: Measures combined I/O + CPU performance
- **Memory Allocation**: Benchmarks memory allocation speed
- **Compute Benchmark**: Pure CPU SHA256 hashing performance

### System Metrics

- **CPU**: Usage %, per-state breakdown (user/system/iowait/irq/steal), load averages
- **Memory**: Total/used/free/available, buffers, cached, swap, dirty pages, writeback
- **Disk I/O**: Reads/writes completed, sectors read/written, I/O time, queue depth
- **Network**: RX/TX bytes, packets, errors
- **Processes**: Count, running, blocked
- **Temperatures**: CPU and max system temperature (via hwmon)
- **PSI (Pressure Stall Information)**: CPU/memory/IO pressure metrics
- **VM Stats**: Page faults (minor & major), swap in/out, page in/out
- **File Descriptors**: Allocated vs max

### Output

- **TUI Mode**: Real-time terminal UI with charts and detailed metrics
- **Headless Mode**: Simple logging to stdout
- **CSV Logging**: All metrics logged to CSV for analysis
- **JSONL Logging**: All metrics logged to JSONL for programmatic processing

## Installation

```bash
cargo build --release
# Binary will be at target/release/slow-rs
```

## Usage

```bash
# Run with TUI (default)
./target/release/slow-rs

# Run in headless mode (good for remote sessions or logging)
./target/release/slow-rs --headless

# Custom interval (default 5 seconds)
./target/release/slow-rs -i 10

# Skip I/O benchmark (if you suspect disk is failing)
./target/release/slow-rs --skip-io-bench

# Full options
./target/release/slow-rs --help
```

### Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `-i, --interval` | Seconds between measurements | 5 |
| `-c, --csv-file` | Path to CSV log file | metrics.csv |
| `-j, --jsonl-file` | Path to JSONL log file | metrics.jsonl |
| `-t, --test-file` | Path to I/O test file | /tmp/slowtest.bin |
| `-s, --file-size-mb` | Size of test file in MB | 256 |
| `--history-size` | Data points to keep for plotting | 120 |
| `--headless` | Run without TUI | false |
| `--skip-io-bench` | Skip I/O benchmark | false |

## Interpreting Results

### Signs of Different Problems

**Disk Failure / I/O Issues:**

- Low I/O read/write speeds
- High I/O pressure (io_pressure_some_avg10 > 10%)
- High disk_weighted_io_time_ms
- High cpu_iowait
- Many pgmajfault (major page faults = disk reads)

**Memory Pressure:**

- High mem_pressure values
- Swap usage increasing
- High pswpin/pswpout (swap activity)
- Low mem_available_mb
- High dirty pages or writeback

**CPU Issues:**

- High cpu_steal (VM being throttled)
- High temperatures
- Low compute benchmark scores
- High load averages with low CPU usage (suggests I/O wait)

**Thermal Throttling:**

- Temperatures > 80-90°C
- Decreasing benchmark scores over time
- CPU usage drops while load stays high

**General Software Issues:**

- High process count
- High context_switches
- FD count approaching fd_max

## Analyzing Logged Data

The CSV file can be analyzed with pandas, Excel, or other tools:

```python
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv('metrics.csv')
df['datetime'] = pd.to_datetime(df['datetime'])

# Plot I/O read speed over time
plt.figure(figsize=(12, 6))
plt.plot(df['datetime'], df['io_read_mb_per_sec'])
plt.title('I/O Read Speed Over Time')
plt.ylabel('MB/s')
plt.xlabel('Time')
plt.savefig('io_speed.png')
```

## Requirements

- Linux (uses `/proc` filesystem)
- Rust 1.70+ (for building)

## License

MIT
