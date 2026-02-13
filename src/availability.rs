//! Metric availability tracking for slow-rs.
//!
//! This module tracks which metric sources are available on the system,
//! allowing the UI to show warnings when metrics are missing due to
//! permissions or kernel configuration.

use std::process::Command;

/// Tracks which metric sources are available.
#[derive(Default, Clone, Debug)]
pub struct MetricAvailability {
    /// /proc/pressure/* is available (requires Linux 4.20+ with CONFIG_PSI)
    pub proc_pressure: bool,
    /// jc42 DIMM temperature sensors found
    pub sys_hwmon_dimm: bool,
    /// NVMe temperature sensors found
    pub sys_hwmon_nvme: bool,
    /// perf events accessible (requires CAP_PERFMON or root)
    pub perf_events: bool,
    /// smartctl is available
    pub smartctl: bool,
    /// ipmitool is available (for BMC sensors)
    pub ipmitool: bool,
}

impl MetricAvailability {
    /// Probe all metric sources and return availability status.
    pub fn probe() -> Self {
        Self {
            proc_pressure: std::fs::read_to_string("/proc/pressure/cpu").is_ok(),
            sys_hwmon_dimm: Self::check_dimm_sensors(),
            sys_hwmon_nvme: Self::check_nvme_sensors(),
            perf_events: Self::check_perf_events(),
            smartctl: Self::check_command_available("smartctl"),
            ipmitool: Self::check_command_available("ipmitool"),
        }
    }

    /// Check for jc42 DIMM temperature sensors.
    fn check_dimm_sensors() -> bool {
        if let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let name_path = entry.path().join("name");
                if let Ok(name) = std::fs::read_to_string(&name_path) {
                    if name.trim() == "jc42" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check for NVMe temperature sensors.
    fn check_nvme_sensors() -> bool {
        if let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let name_path = entry.path().join("name");
                if let Ok(name) = std::fs::read_to_string(&name_path) {
                    if name.trim() == "nvme" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if perf events are accessible.
    fn check_perf_events() -> bool {
        std::fs::read_to_string("/proc/sys/kernel/perf_event_paranoid")
            .map(|s| s.trim().parse::<i32>().unwrap_or(2) <= 1)
            .unwrap_or(false)
    }

    /// Check if a command is available in PATH.
    fn check_command_available(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Generate warnings for unavailable metrics.
    pub fn get_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if !self.proc_pressure {
            warnings.push("PSI unavailable (requires Linux 4.20+ with CONFIG_PSI)".into());
        }
        if !self.sys_hwmon_dimm {
            warnings.push("RAM temp sensors not found (no jc42 hwmon devices)".into());
        }
        if !self.sys_hwmon_nvme {
            warnings.push("NVMe temp sensors not found".into());
        }
        if !self.perf_events && !Self::has_elevated_privileges() {
            warnings.push("Perf events restricted (run with sudo for full metrics)".into());
        }
        if !self.smartctl {
            warnings.push("smartctl not found (install smartmontools for disk health)".into());
        }
        if !self.ipmitool && Self::has_elevated_privileges() {
            warnings.push("ipmitool not found (install for BMC/IPMI sensors)".into());
        }

        warnings
    }

    /// Check if running with elevated privileges.
    pub fn has_elevated_privileges() -> bool {
        unsafe { libc::geteuid() == 0 }
    }

    /// Check if we have passwordless sudo access.
    pub fn has_sudo_access() -> bool {
        Command::new("sudo")
            .args(["-n", "true"])
            .output()
            .map(|s| s.status.success())
            .unwrap_or(false)
    }
}
