mod api;
mod collector;
mod config;
mod database;

use std::sync::Arc;
use tokio::sync::RwLock;

use collector::SystemMetrics;

use std::collections::HashMap;
use std::time::Instant;

use sysinfo::{Disks, Networks, System};
use tokio::time::{Duration, MissedTickBehavior, interval, sleep};

use config::AppConfig;
use database::GreptimeClient;

#[tokio::main]
async fn main() {
    println!("Starting Rust Server Monitoring Agent...");

    // Load application configuration
    let config = AppConfig::from_env();

    // Initialize GreptimeDB client
    let db_client = Arc::new(GreptimeClient::new(&config));

    // Shared state for API and monitoring task
    let shared_metrics = Arc::new(RwLock::new(SystemMetrics {
        cpu_usage: 0.0,
        memory_used: 0,
        memory_total: 0,
        disks: Vec::new(),
        networks: Vec::new(),
    }));

    // Create Axum router
    let app_state = api::AppState {
    metrics: shared_metrics.clone(),
    db: db_client.clone(),
};

let app = api::create_router(app_state);

    // Bind HTTP server to port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind API server");

    println!("API server listening on http://localhost:3000");

    // Run API server in a separate Tokio task
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            eprintln!("API server error: {error}");
        }
    });

    println!("GreptimeDB URL: {}", config.greptime_url);
    println!("Database: {}", config.database);

    // Initialize system collectors
    let mut system = System::new();
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();

    // Establish initial CPU measurement baseline
    system.refresh_cpu_usage();
    sleep(Duration::from_secs(1)).await;

    // Store previous cumulative network readings
    let mut previous_network: HashMap<String, (u64, u64, Instant)> = HashMap::new();

    // Create a five-second monitoring interval
    let mut ticker = interval(Duration::from_secs(5));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    println!("Monitoring started. Press Ctrl+C to stop.");

    loop {
        tokio::select! {
                    _ = ticker.tick() => {
                        // Collect system metrics
                        let metrics = collector::collect_metrics(
                            &mut system,
                            &mut disks,
                            &mut networks,
                        );

                        // Update the latest metrics for API requests
        {
            let mut shared = shared_metrics.write().await;
            *shared = metrics.clone();
        }

                        println!("\n========== SERVER METRICS ==========");

                        // CPU
                        println!("\nCPU:");
                        println!("Usage: {:.2}%", metrics.cpu_usage);

                        // Memory
                        println!("\nMEMORY:");
                        println!(
                            "Used: {:.2} GB",
                            metrics.memory_used as f64 / 1_073_741_824.0
                        );

                        println!(
                            "Total: {:.2} GB",
                            metrics.memory_total as f64 / 1_073_741_824.0
                        );

                        // Disk
                        println!("\nDISK:");

                        for disk in &metrics.disks {
                            println!("Name: {}", disk.name);
                            println!("Mount: {}", disk.mount_point);

                            println!(
                                "Total: {:.2} GB",
                                disk.total_space as f64 / 1_073_741_824.0
                            );

                            println!(
                                "Used: {:.2} GB",
                                disk.used_space as f64 / 1_073_741_824.0
                            );

                            println!("Usage: {:.2}%", disk.usage_percent);

                            println!(
                                "Available: {:.2} GB",
                                disk.available_space as f64 / 1_073_741_824.0
                            );
                        }

                        // Network
                        println!("\nNETWORK:");

                        for network in &metrics.networks {
                            let now = Instant::now();

                            let (rx_rate, tx_rate) =
                                if let Some((prev_rx, prev_tx, prev_time)) =
                                    previous_network.get(&network.interface)
                                {
                                    let elapsed =
                                        now.duration_since(*prev_time).as_secs_f64();

                                    if elapsed > 0.0 {
                                        let rx = network.total_received_bytes
                                            .saturating_sub(*prev_rx);

                                        let tx = network.total_transmitted_bytes
                                            .saturating_sub(*prev_tx);

                                        (
                                            rx as f64 / elapsed / 1_048_576.0,
                                            tx as f64 / elapsed / 1_048_576.0,
                                        )
                                    } else {
                                        (0.0, 0.0)
                                    }
                                } else {
                                    (0.0, 0.0)
                                };

                            // Save current readings for next calculation
                            previous_network.insert(
                                network.interface.clone(),
                                (
                                    network.total_received_bytes,
                                    network.total_transmitted_bytes,
                                    now,
                                ),
                            );

                            println!("Interface: {}", network.interface);
                            println!("Download: {:.3} MB/s", rx_rate);
                            println!("Upload: {:.3} MB/s", tx_rate);

                            println!(
                                "Received this interval: {} bytes",
                                network.received_bytes
                            );

                            println!(
                                "Transmitted this interval: {} bytes",
                                network.transmitted_bytes
                            );
                        }

                        // Insert metrics into GreptimeDB
                        println!("\nDATABASE:");

                        match db_client.insert_metrics(&metrics).await {
                            Ok(()) => {
                                println!("Metrics successfully sent to GreptimeDB.");
                            }
                            Err(error) => {
                                eprintln!("Failed to insert metrics: {error}");
                            }
                        }

                        println!("\n====================================");
                    }

                    result = tokio::signal::ctrl_c() => {
                        match result {
                            Ok(()) => println!("\nMonitoring agent stopped."),
                            Err(error) => {
                                eprintln!("Shutdown signal error: {error}");
                            }
                        }

                        break;
                    }
                }
    }
}
