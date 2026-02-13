//! Threshold definitions for slow-rs.
//!
//! This module defines severity levels and threshold values for
//! determining when metrics should trigger warnings or critical alerts.

/// Severity level for a metric.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Severity {
    /// Normal operating range
    #[default]
    Normal,
    /// Approaching problematic levels
    Warning,
    /// Critical - immediate attention needed
    Critical,
}

/// Threshold configuration for all monitored metrics.
#[derive(Clone, Debug)]
pub struct Thresholds {
    /// I/O pressure warning threshold (%)
    pub io_pressure_warning: f64,
    /// I/O pressure critical threshold (%)
    pub io_pressure_critical: f64,
    /// CPU usage warning threshold (%)
    pub cpu_usage_warning: f32,
    /// CPU usage critical threshold (%)
    pub cpu_usage_critical: f32,
    /// Memory available warning threshold (MB)
    pub memory_available_warning_mb: u64,
    /// Memory available critical threshold (MB)
    pub memory_available_critical_mb: u64,
    /// CPU temperature warning threshold (C)
    pub cpu_temp_warning: f64,
    /// CPU temperature critical threshold (C)
    pub cpu_temp_critical: f64,
    /// DIMM temperature warning threshold (C)
    pub dimm_temp_warning: f64,
    /// DIMM temperature critical threshold (C)
    pub dimm_temp_critical: f64,
    /// Disk temperature warning threshold (C)
    pub disk_temp_warning: f64,
    /// Disk temperature critical threshold (C)
    pub disk_temp_critical: f64,
    /// Memory pressure warning threshold (%)
    pub mem_pressure_warning: f64,
    /// Memory pressure critical threshold (%)
    pub mem_pressure_critical: f64,
    /// I/O wait percentage warning threshold
    pub iowait_warning: f64,
    /// I/O wait percentage critical threshold
    pub iowait_critical: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            io_pressure_warning: 10.0,
            io_pressure_critical: 25.0,
            cpu_usage_warning: 80.0,
            cpu_usage_critical: 95.0,
            memory_available_warning_mb: 1024,
            memory_available_critical_mb: 256,
            cpu_temp_warning: 75.0,
            cpu_temp_critical: 85.0,
            dimm_temp_warning: 70.0,
            dimm_temp_critical: 80.0,
            disk_temp_warning: 50.0,
            disk_temp_critical: 60.0,
            mem_pressure_warning: 10.0,
            mem_pressure_critical: 25.0,
            iowait_warning: 20.0,
            iowait_critical: 40.0,
        }
    }
}

impl Thresholds {
    /// Evaluate I/O pressure severity.
    pub fn io_pressure_severity(&self, value: f64) -> Severity {
        if value >= self.io_pressure_critical {
            Severity::Critical
        } else if value >= self.io_pressure_warning {
            Severity::Warning
        } else {
            Severity::Normal
        }
    }

    /// Evaluate CPU usage severity.
    pub fn cpu_usage_severity(&self, value: f32) -> Severity {
        if value >= self.cpu_usage_critical {
            Severity::Critical
        } else if value >= self.cpu_usage_warning {
            Severity::Warning
        } else {
            Severity::Normal
        }
    }

    /// Evaluate memory available severity (inverted - low is bad).
    pub fn memory_available_severity(&self, value_mb: u64) -> Severity {
        if value_mb <= self.memory_available_critical_mb {
            Severity::Critical
        } else if value_mb <= self.memory_available_warning_mb {
            Severity::Warning
        } else {
            Severity::Normal
        }
    }

    /// Evaluate CPU temperature severity.
    pub fn cpu_temp_severity(&self, value: f64) -> Severity {
        if value >= self.cpu_temp_critical {
            Severity::Critical
        } else if value >= self.cpu_temp_warning {
            Severity::Warning
        } else {
            Severity::Normal
        }
    }

    /// Evaluate DIMM temperature severity.
    pub fn dimm_temp_severity(&self, value: f64) -> Severity {
        if value >= self.dimm_temp_critical {
            Severity::Critical
        } else if value >= self.dimm_temp_warning {
            Severity::Warning
        } else {
            Severity::Normal
        }
    }

    /// Evaluate disk temperature severity.
    pub fn disk_temp_severity(&self, value: f64) -> Severity {
        if value >= self.disk_temp_critical {
            Severity::Critical
        } else if value >= self.disk_temp_warning {
            Severity::Warning
        } else {
            Severity::Normal
        }
    }

    /// Evaluate memory pressure severity.
    pub fn mem_pressure_severity(&self, value: f64) -> Severity {
        if value >= self.mem_pressure_critical {
            Severity::Critical
        } else if value >= self.mem_pressure_warning {
            Severity::Warning
        } else {
            Severity::Normal
        }
    }

    /// Evaluate I/O wait percentage severity.
    pub fn iowait_severity(&self, iowait_pct: f64) -> Severity {
        if iowait_pct >= self.iowait_critical {
            Severity::Critical
        } else if iowait_pct >= self.iowait_warning {
            Severity::Warning
        } else {
            Severity::Normal
        }
    }
}
