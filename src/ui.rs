//! Terminal User Interface for slow-rs.
//!
//! This module provides a real-time dashboard using `ratatui` that displays:
//!
//! - Status bar with current metrics summary
//! - Four charts showing key metrics over time
//! - Detailed metrics panels at the bottom
//!
//! # Controls
//!
//! - `q` or `Esc`: Quit
//! - `Up`/`Down`: Scroll (reserved for future use)

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::app::App;
use crate::metrics::Metrics;

/// Run the TUI event loop.
///
/// This takes ownership of the App and terminal, running until the user
/// presses `q` or `Esc`, or the `running` flag is set to false.
///
/// # Arguments
///
/// * `app` - Application instance
/// * `running` - Atomic flag to signal shutdown
/// * `interval` - Time between metric collections
pub fn run(
    mut app: App,
    running: Arc<AtomicBool>,
    interval: Duration,
) -> std::io::Result<()> {
    let history_size = app.config.history_size;
    
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut last_collection = Instant::now();
    let mut _scroll_offset = 0usize;

    // Initial collection
    if let Ok(metrics) = app.collect_metrics() {
        add_metrics(&mut app.metrics_history, metrics, history_size);
    }

    while running.load(Ordering::Relaxed) {
        // Check for input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            running.store(false, Ordering::Relaxed);
                        }
                        KeyCode::Up => {
                            _scroll_offset = _scroll_offset.saturating_sub(1);
                        }
                        KeyCode::Down => {
                            _scroll_offset = _scroll_offset.saturating_add(1);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Collect metrics at interval
        if last_collection.elapsed() >= interval {
            if let Ok(metrics) = app.collect_metrics() {
                add_metrics(&mut app.metrics_history, metrics, history_size);
            }
            last_collection = Instant::now();
        }

        // Draw UI
        terminal.draw(|f| draw_ui(f, &app.metrics_history))?;
    }

    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

/// Add metrics to history, maintaining max size.
fn add_metrics(history: &mut VecDeque<Metrics>, metrics: Metrics, max_size: usize) {
    if history.len() >= max_size {
        history.pop_front();
    }
    history.push_back(metrics);
}

/// Main UI drawing function.
fn draw_ui(f: &mut Frame, metrics_history: &VecDeque<Metrics>) {
    let size = f.area();

    // Main layout: status bar, charts, details
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Status bar
            Constraint::Min(20),    // Charts
            Constraint::Length(12), // Detailed metrics
        ])
        .split(size);

    draw_status_bar(f, metrics_history, main_chunks[0]);
    draw_charts(f, metrics_history, main_chunks[1]);
    draw_details(f, metrics_history, main_chunks[2]);
}

/// Draw the top status bar.
fn draw_status_bar(f: &mut Frame, metrics_history: &VecDeque<Metrics>, area: Rect) {
    let status_text = if let Some(m) = metrics_history.back() {
        format!(
            " 📊 slow-rs | {} | CPU: {:.1}% | Mem: {}/{} MB | Load: {:.2} {:.2} {:.2} | Samples: {} | [q]uit",
            m.datetime,
            m.cpu_usage_percent,
            m.mem_used_mb,
            m.mem_total_mb,
            m.load_avg_1,
            m.load_avg_5,
            m.load_avg_15,
            metrics_history.len()
        )
    } else {
        " 📊 slow-rs | Collecting initial metrics... | [q]uit".to_string()
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title("Status"));

    f.render_widget(status, area);
}

/// Draw the 2x2 grid of charts.
fn draw_charts(f: &mut Frame, metrics_history: &VecDeque<Metrics>, area: Rect) {
    if metrics_history.is_empty() {
        let loading = Paragraph::new("Waiting for data...")
            .block(Block::default().borders(Borders::ALL).title("Charts"));
        f.render_widget(loading, area);
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bot_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    // Chart 1: I/O Read Speed
    draw_line_chart(
        f,
        metrics_history,
        top_cols[0],
        "I/O Read (MB/s)",
        |m| m.io_read_mb_per_sec.unwrap_or(0.0),
        Color::Cyan,
    );

    // Chart 2: CPU Usage
    draw_line_chart(
        f,
        metrics_history,
        top_cols[1],
        "CPU Usage (%)",
        |m| m.cpu_usage_percent as f64,
        Color::Yellow,
    );

    // Chart 3: Memory Available
    draw_line_chart(
        f,
        metrics_history,
        bot_cols[0],
        "Memory Available (MB)",
        |m| m.mem_available_mb as f64,
        Color::Green,
    );

    // Chart 4: I/O Pressure
    draw_line_chart(
        f,
        metrics_history,
        bot_cols[1],
        "I/O Pressure (avg10)",
        |m| m.io_pressure_some_avg10.unwrap_or(0.0),
        Color::Red,
    );
}

/// Draw a single line chart.
fn draw_line_chart<F>(
    f: &mut Frame,
    metrics_history: &VecDeque<Metrics>,
    area: Rect,
    title: &str,
    value_fn: F,
    color: Color,
) where
    F: Fn(&Metrics) -> f64,
{
    let data: Vec<(f64, f64)> = metrics_history
        .iter()
        .enumerate()
        .map(|(i, m)| (i as f64, value_fn(m)))
        .collect();

    if data.is_empty() {
        return;
    }

    let min_y = data.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
    let max_y = data
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max);

    let y_range = if (max_y - min_y).abs() < 0.001 {
        (min_y - 1.0, max_y + 1.0)
    } else {
        (min_y * 0.9, max_y * 1.1)
    };

    let dataset = Dataset::default()
        .name(title)
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))
        .data(&data);

    let chart = Chart::new(vec![dataset])
        .block(Block::default().borders(Borders::ALL).title(title))
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, data.len() as f64]),
        )
        .y_axis(
            Axis::default()
                .title("")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::raw(format!("{:.1}", y_range.0)),
                    Span::raw(format!("{:.1}", y_range.1)),
                ])
                .bounds([y_range.0, y_range.1]),
        );

    f.render_widget(chart, area);
}

/// Draw the bottom detail panels.
fn draw_details(f: &mut Frame, metrics_history: &VecDeque<Metrics>, area: Rect) {
    let latest = match metrics_history.back() {
        Some(m) => m,
        None => return,
    };

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Column 1: Benchmarks
    let bench_items = vec![
        ListItem::new(format!(
            "Read:   {:>7.1} MB/s",
            latest.io_read_mb_per_sec.unwrap_or(0.0)
        )),
        ListItem::new(format!(
            "Write:  {:>7.1} MB/s",
            latest.io_write_mb_per_sec.unwrap_or(0.0)
        )),
        ListItem::new(format!(
            "SHA256: {:>7.1} ms",
            latest.sha256_duration_ms.unwrap_or(0.0)
        )),
        ListItem::new(format!("Alloc:  {:>7.1} ms", latest.memory_alloc_duration_ms)),
        ListItem::new(format!("Compute:{:>7.1} ms", latest.compute_duration_ms)),
    ];
    let bench_list = List::new(bench_items)
        .block(Block::default().borders(Borders::ALL).title("Benchmarks"));
    f.render_widget(bench_list, cols[0]);

    // Column 2: Memory
    let mem_items = vec![
        ListItem::new(format!("Used:     {:>6} MB", latest.mem_used_mb)),
        ListItem::new(format!("Available:{:>6} MB", latest.mem_available_mb)),
        ListItem::new(format!("Buffers:  {:>6} MB", latest.mem_buffers_mb)),
        ListItem::new(format!("Cached:   {:>6} MB", latest.mem_cached_mb)),
        ListItem::new(format!("Swap:     {:>6} MB", latest.swap_used_mb)),
        ListItem::new(format!("Dirty:    {:>6} MB", latest.dirty_mb)),
    ];
    let mem_list =
        List::new(mem_items).block(Block::default().borders(Borders::ALL).title("Memory"));
    f.render_widget(mem_list, cols[1]);

    // Column 3: I/O & Pressure
    let io_items = vec![
        ListItem::new(format!(
            "IO Press: {:>5.1}%",
            latest.io_pressure_some_avg10.unwrap_or(0.0)
        )),
        ListItem::new(format!(
            "Mem Press:{:>5.1}%",
            latest.mem_pressure_some_avg10.unwrap_or(0.0)
        )),
        ListItem::new(format!(
            "CPU Press:{:>5.1}%",
            latest.cpu_pressure_some_avg10.unwrap_or(0.0)
        )),
        ListItem::new(format!("IOWait:   {:>6}", latest.cpu_iowait)),
        ListItem::new(format!("MajFaults:{:>6}", latest.pgmajfault)),
        ListItem::new(format!("SwapIn:   {:>6}", latest.pswpin)),
    ];
    let io_list =
        List::new(io_items).block(Block::default().borders(Borders::ALL).title("Pressure/IO"));
    f.render_widget(io_list, cols[2]);

    // Column 4: System
    let sys_items = vec![
        ListItem::new(format!("Procs:    {:>6}", latest.process_count)),
        ListItem::new(format!("Running:  {:>6}", latest.procs_running)),
        ListItem::new(format!("Blocked:  {:>6}", latest.procs_blocked)),
        ListItem::new(format!(
            "CPU Temp: {:>5.1}°C",
            latest.cpu_temp_celsius.unwrap_or(0.0)
        )),
        ListItem::new(format!(
            "Max Temp: {:>5.1}°C",
            latest.max_temp_celsius.unwrap_or(0.0)
        )),
        ListItem::new(format!("FDs:      {:>6}", latest.fd_allocated)),
    ];
    let sys_list =
        List::new(sys_items).block(Block::default().borders(Borders::ALL).title("System"));
    f.render_widget(sys_list, cols[3]);
}

/// Run in headless mode (no TUI, just logging to stdout).
///
/// # Arguments
///
/// * `app` - Application instance
/// * `running` - Atomic flag to signal shutdown
/// * `interval` - Time between metric collections
pub fn run_headless(
    mut app: App,
    running: Arc<AtomicBool>,
    interval: Duration,
) -> std::io::Result<()> {
    let csv_file = app.config.csv_file.clone();
    let history_size = app.config.history_size;
    
    println!("slow-rs - System Slowness Diagnostic Monitor");
    println!("=============================================");
    println!("Logging to: {}", csv_file);
    println!("Interval: {} seconds", interval.as_secs());
    println!("Press Ctrl+C to stop.\n");

    while running.load(Ordering::Relaxed) {
        let metrics = app.collect_metrics()?;

        // Print summary line
        println!(
            "[{}] CPU: {:5.1}% | Mem: {:6}/{:6} MB | Load: {:5.2} {:5.2} {:5.2} | Read: {:7.1} MB/s | IOPress: {:5.1}%",
            metrics.datetime,
            metrics.cpu_usage_percent,
            metrics.mem_used_mb,
            metrics.mem_total_mb,
            metrics.load_avg_1,
            metrics.load_avg_5,
            metrics.load_avg_15,
            metrics.io_read_mb_per_sec.unwrap_or(0.0),
            metrics.io_pressure_some_avg10.unwrap_or(0.0),
        );

        add_metrics(&mut app.metrics_history, metrics, history_size);
        std::thread::sleep(interval);
    }

    println!("\nStopped. Data logged to {}", csv_file);
    Ok(())
}
