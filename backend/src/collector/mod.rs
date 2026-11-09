use serde::Serialize;
use sysinfo::{Disks, Networks, System};

// Represents disk information
#[derive(Clone, Serialize)]
pub struct DiskMetrics {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: f64,
}

#[derive(Clone, Serialize)]
pub struct NetworkMetrics {
    pub interface: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
    pub total_received_bytes: u64,
    pub total_transmitted_bytes: u64,
}

#[derive(Clone, Serialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disks: Vec<DiskMetrics>,
    pub networks: Vec<NetworkMetrics>,
}

pub fn collect_metrics(
    system: &mut System,
    disks: &mut Disks,
    networks: &mut Networks,
) -> SystemMetrics {

    // Refresh CPU and memory
    system.refresh_cpu_usage();
    system.refresh_memory();

    // Refresh disk and network information
    disks.refresh(true);
    networks.refresh(true);

    // Collect disk metrics
    let disk_metrics = disks
        .iter()
        .map(|disk| {
            let total_space = disk.total_space();
            let available_space = disk.available_space();

            let used_space = total_space.saturating_sub(available_space);

            let usage_percent = if total_space > 0 {
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };

            DiskMetrics {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space,
                available_space,
                used_space,
                usage_percent,
            }
        })
        .collect();

    // Collect network metrics
    let network_metrics = networks
        .iter()
        .map(|(interface_name, data)| {
            NetworkMetrics {
                interface: interface_name.to_string(),
                received_bytes: data.received(),
                transmitted_bytes: data.transmitted(),
                total_received_bytes: data.total_received(),
                total_transmitted_bytes: data.total_transmitted(),
            }
        })
        .collect();

    SystemMetrics {
        cpu_usage: system.global_cpu_usage(),
        memory_used: system.used_memory(),
        memory_total: system.total_memory(),
        disks: disk_metrics,
        networks: network_metrics,
    }
}