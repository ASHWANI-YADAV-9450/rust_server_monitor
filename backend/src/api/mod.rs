
use std::sync::Arc;
use crate::database::GreptimeClient;

use axum::{
    extract::State,
    routing::get,
    Json,
    Router,
};

use axum::extract::Query;
use tokio::sync::RwLock;

use crate::collector::SystemMetrics;

pub type SharedMetrics = Arc<RwLock<SystemMetrics>>;

#[derive(serde::Deserialize)]
struct HistoryParams {
    limit: Option<usize>,
}

#[derive(Clone)]
pub struct AppState {
    pub metrics: SharedMetrics,
    pub db: Arc<GreptimeClient>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/metrics/cpu", get(get_cpu_metrics))
        .route("/metrics/memory", get(get_memory_metrics))
        .route("/metrics/disk", get(get_disk_metrics))
        .route("/metrics/network", get(get_network_metrics))
        .route("/metrics/cpu/history", get(get_cpu_history))
        .route("/metrics/memory/history", get(get_memory_history))
        .route("/metrics/disk/history", get(get_disk_history))
        .route("/metrics/network/history", get(get_network_history))
        .with_state(state)
}

async fn health_check() -> Json<serde_json::Value> {    Json(serde_json::json!({
        "status": "healthy",
        "service": "rust-server-monitor"
    }))
}

// Return all system metrics
async fn get_metrics(
    State(state): State<AppState>
) -> Json<SystemMetrics> {
    let metrics = state.metrics.read().await;
    Json(metrics.clone())
}

// Return CPU metrics only
async fn get_cpu_metrics(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let metrics = state.metrics.read().await;

    Json(serde_json::json!({
        "cpu_usage": metrics.cpu_usage
    }))
}


async fn get_memory_metrics(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let metrics = state.metrics.read().await;

    Json(serde_json::json!({
        "memory_used": metrics.memory_used,
        "memory_total": metrics.memory_total,
        "memory_usage_percent": if metrics.memory_total > 0 {
            (metrics.memory_used as f64 / metrics.memory_total as f64) * 100.0
        } else {
            0.0
        }
    }))
}


async fn get_disk_metrics(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let metrics = state.metrics.read().await;

    Json(serde_json::json!({
        "disks": metrics.disks.iter().map(|disk| {
            serde_json::json!({
                "name": disk.name,
                "mount_point": disk.mount_point,
                "total_space": disk.total_space,
                "used_space": disk.used_space,
                "available_space": disk.available_space,
                "usage_percent": disk.usage_percent
            })
        }).collect::<Vec<_>>()
    }))
}

async fn get_disk_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let limit = params.limit.unwrap_or(60).clamp(1, 1000);

    match state.db.query_disk_history(limit).await {
        Ok(history) => Ok(Json(history)),
        Err(error) => {
            tracing::error!("Failed to fetch disk history: {}", error);

            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch disk history".to_string(),
            ))
        }
    }
}

async fn get_network_metrics(
    State(state): State<AppState>
) -> Json<serde_json::Value> {
    let metrics = state.metrics.read().await;

    Json(serde_json::json!({
        "networks": metrics.networks.iter().map(|network| {
            serde_json::json!({
                "interface": network.interface,
                "received_bytes": network.received_bytes,
                "transmitted_bytes": network.transmitted_bytes,
                "total_received_bytes": network.total_received_bytes,
                "total_transmitted_bytes": network.total_transmitted_bytes
            })
        }).collect::<Vec<_>>()
    }))
}

async fn get_memory_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let limit = params.limit.unwrap_or(60).clamp(1, 1000);

    match state.db.query_memory_history(limit).await {
        Ok(history) => Ok(Json(history)),
        Err(error) => {
            tracing::error!("Failed to fetch memory history: {}", error);

            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch memory history".to_string(),
            ))
        }
    }
}

async fn get_network_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let limit = params.limit.unwrap_or(60).clamp(1, 1000);

    match state.db.query_network_history(limit).await {
        Ok(history) => Ok(Json(history)),
        Err(error) => {
            tracing::error!("Failed to fetch network history: {}", error);

            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch network history".to_string(),
            ))
        }
    }
}

async fn get_cpu_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let limit = params.limit.unwrap_or(60).clamp(1, 1000);

    match state.db.query_cpu_history(limit).await {
        Ok(history) => Ok(Json(history)),
        Err(error) => {
            tracing::error!("Failed to fetch CPU history: {}", error);

            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch CPU history".to_string(),
            ))
        }
    }
}