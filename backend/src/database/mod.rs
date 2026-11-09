use crate::collector::SystemMetrics;
use crate::config::AppConfig;

pub struct GreptimeClient {
    client: reqwest::Client,
    endpoint: String,
    database: String,
    host: String,
}

impl GreptimeClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: format!("{}/v1/sql", config.greptime_url.trim_end_matches('/')),
            database: config.database.clone(),
            host: config.host.clone(),
        }
    }

    pub async fn insert_metrics(
        &self,
        metrics: &SystemMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Select the root disk to avoid counting duplicate APFS volumes.
        let disk = metrics
            .disks
            .iter()
            .find(|disk| disk.mount_point == "/")
            .or_else(|| metrics.disks.first());

        let disk_used = disk.map(|d| d.used_space).unwrap_or(0);
        let disk_total = disk.map(|d| d.total_space).unwrap_or(0);

        // Aggregate bytes received/transmitted during this collection interval.
        let network_rx: u128 = metrics
            .networks
            .iter()
            .map(|network| network.received_bytes as u128)
            .sum();

        let network_tx: u128 = metrics
            .networks
            .iter()
            .map(|network| network.transmitted_bytes as u128)
            .sum();

        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string();

        // Escape single quotes in the host name.
        let host = self.host.replace('\'', "''");

        let sql = format!(
            "INSERT INTO server_metrics \
            (host, cpu_usage, memory_used, memory_total, \
            disk_used, disk_total, network_rx, network_tx, ts) \
            VALUES ('{}', {}, {}, {}, {}, {}, {}, {}, '{}')",
            host,
            metrics.cpu_usage,
            metrics.memory_used,
            metrics.memory_total,
            disk_used,
            disk_total,
            network_rx,
            network_tx,
            timestamp
        );

        let url = format!("{}?db={}", self.endpoint, self.database);

        let response = self
            .client
            .post(&url)
            .form(&[("sql", sql)])
            .send()
            .await?
            .error_for_status()?;

        let response_body = response.text().await?;

        println!("GreptimeDB insert response: {}", response_body);

        Ok(())
    }

    pub async fn query_cpu_history(
        &self,
        limit: usize,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let host = self.host.replace('\'', "''");

        let sql = format!(
            "SELECT cpu_usage, ts \
             FROM server_metrics \
             WHERE host = '{}' \
             ORDER BY ts DESC \
             LIMIT {}",
            host, limit
        );

        let url = format!("{}?db={}", self.endpoint, self.database);

        let response = self
            .client
            .post(&url)
            .form(&[("sql", sql)])
            .send()
            .await?
            .error_for_status()?;

        let result = response.json::<serde_json::Value>().await?;

        let rows = result["output"][0]["records"]["rows"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let history: Vec<serde_json::Value> = rows
            .iter()
            .filter_map(|row| {
                let values = row.as_array()?;

                Some(serde_json::json!({
                    "cpu_usage": values.first()?.as_f64()?,
                    "timestamp": values.get(1)?.as_i64()?
                }))
            })
            .collect();

        Ok(serde_json::json!({
            "host": self.host,
            "count": history.len(),
            "data": history
        }))
    }

    pub async fn query_disk_history(
        &self,
        limit: usize,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let host = self.host.replace('\'', "''");

        let sql = format!(
            "SELECT disk_used, disk_total, ts \
         FROM server_metrics \
         WHERE host = '{}' \
         ORDER BY ts DESC \
         LIMIT {}",
            host, limit
        );

        let url = format!("{}?db={}", self.endpoint, self.database);

        let response = self
            .client
            .post(&url)
            .form(&[("sql", sql)])
            .send()
            .await?
            .error_for_status()?;

        let result = response.json::<serde_json::Value>().await?;

        let rows = result["output"][0]["records"]["rows"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let history: Vec<serde_json::Value> = rows
            .iter()
            .filter_map(|row| {
                let values = row.as_array()?;

                let disk_used = values.first()?.as_f64()?;
                let disk_total = values.get(1)?.as_f64()?;
                let timestamp = values.get(2)?.as_i64()?;

                let usage_percent = if disk_total > 0.0 {
                    ((disk_used / disk_total) * 100.0 * 100.0).round() / 100.0
                } else {
                    0.0
                };

                Some(serde_json::json!({
                    "disk_used": disk_used,
                    "disk_total": disk_total,
                    "disk_usage_percent": usage_percent,
                    "timestamp": timestamp
                }))
            })
            .collect();

        Ok(serde_json::json!({
            "host": self.host,
            "count": history.len(),
            "data": history
        }))
    }

    pub async fn query_network_history(
        &self,
        limit: usize,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let host = self.host.replace('\'', "''");

        let sql = format!(
            "SELECT network_rx, network_tx, ts \
         FROM server_metrics \
         WHERE host = '{}' \
         ORDER BY ts DESC \
         LIMIT {}",
            host, limit
        );

        let url = format!("{}?db={}", self.endpoint, self.database);

        let response = self
            .client
            .post(&url)
            .form(&[("sql", sql)])
            .send()
            .await?
            .error_for_status()?;

        let result = response.json::<serde_json::Value>().await?;

        let rows = result["output"][0]["records"]["rows"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        // First extract the values from GreptimeDB.
        let records: Vec<(f64, f64, i64)> = rows
            .iter()
            .filter_map(|row| {
                let values = row.as_array()?;

                let network_rx = values.first()?.as_f64()?;
                let network_tx = values.get(1)?.as_f64()?;
                let timestamp = values.get(2)?.as_i64()?;

                Some((network_rx, network_tx, timestamp))
            })
            .collect();

        // Calculate network speed using actual elapsed time.
        let history: Vec<serde_json::Value> = records
            .iter()
            .enumerate()
            .map(|(index, &(network_rx, network_tx, timestamp))| {
                let speeds = records
                    .get(index + 1)
                    .and_then(|&(_, _, previous_timestamp)| {
                        let elapsed_seconds = (timestamp - previous_timestamp) as f64 / 1000.0;

                        if elapsed_seconds <= 0.0 {
                            return None;
                        }

                        let rx_speed = network_rx / elapsed_seconds / 1_048_576.0;

                        let tx_speed = network_tx / elapsed_seconds / 1_048_576.0;

                        Some((rx_speed, tx_speed))
                    });

                let (rx_speed, tx_speed) = speeds
                    .map(|(rx, tx)| {
                        (
                            (rx * 1000.0).round() / 1000.0,
                            (tx * 1000.0).round() / 1000.0,
                        )
                    })
                    .unwrap_or((0.0, 0.0));

                serde_json::json!({
                    "network_rx_bytes": network_rx,
                    "network_tx_bytes": network_tx,
                    "rx_speed_mb_per_sec": rx_speed,
                    "tx_speed_mb_per_sec": tx_speed,
                    "timestamp": timestamp
                })
            })
            .collect();

        Ok(serde_json::json!({
            "host": self.host,
            "count": history.len(),
            "data": history
        }))
    }

    pub async fn query_memory_history(
        &self,
        limit: usize,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let host = self.host.replace('\'', "''");

        let sql = format!(
            "SELECT memory_used, memory_total, ts \
         FROM server_metrics \
         WHERE host = '{}' \
         ORDER BY ts DESC \
         LIMIT {}",
            host, limit
        );

        let url = format!("{}?db={}", self.endpoint, self.database);

        let response = self
            .client
            .post(&url)
            .form(&[("sql", sql)])
            .send()
            .await?
            .error_for_status()?;

        let result = response.json::<serde_json::Value>().await?;

        let rows = result["output"][0]["records"]["rows"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let history: Vec<serde_json::Value> = rows
            .iter()
            .filter_map(|row| {
                let values = row.as_array()?;

                let memory_used = values.first()?.as_f64()?;
                let memory_total = values.get(1)?.as_f64()?;
                let timestamp = values.get(2)?.as_i64()?;

                let usage_percent = if memory_total > 0.0 {
                    ((memory_used / memory_total) * 100.0 * 100.0).round() / 100.0
                } else {
                    0.0
                };

                Some(serde_json::json!({
                    "memory_used": memory_used,
                    "memory_total": memory_total,
                    "memory_usage_percent": usage_percent,
                    "timestamp": timestamp
                }))
            })
            .collect();

        Ok(serde_json::json!({
            "host": self.host,
            "count": history.len(),
            "data": history
        }))
    }
}
