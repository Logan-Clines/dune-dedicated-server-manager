use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::scheduler::Task;
use crate::store::TaskTrigger;

use super::{AppState, VERSION};

pub async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "dune-server-service",
        "version": VERSION,
    }))
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub version: &'static str,
    pub now: String,
}

pub async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        ok: true,
        version: VERSION,
        now: Utc::now().to_rfc3339(),
    })
}

#[derive(Debug, Deserialize)]
pub struct RunsQuery {
    pub limit: Option<u32>,
    pub task: Option<String>,
}

pub async fn list_runs(
    State(state): State<AppState>,
    Query(q): Query<RunsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let runs = state
        .store
        .list_runs(q.limit.unwrap_or(50), q.task.as_deref())?;
    Ok(Json(runs))
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub limit: Option<u32>,
    #[serde(rename = "runId")]
    pub run_id: Option<i64>,
}

pub async fn list_logs(
    State(state): State<AppState>,
    Query(q): Query<LogsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let logs = state.store.list_logs(q.limit.unwrap_or(200), q.run_id)?;
    Ok(Json(logs))
}

#[derive(Debug, Deserialize)]
pub struct TriggerRequest {
    pub task: String,
}

pub async fn trigger(
    State(state): State<AppState>,
    Json(req): Json<TriggerRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let tasks: Vec<Arc<dyn Task>> = crate::tasks::build_all(state.env.clone());
    let task = tasks
        .into_iter()
        .find(|t| t.id() == req.task)
        .ok_or_else(|| ApiError::not_found(format!("unknown task: {}", req.task)))?;

    let runner = state.runner.clone();
    tokio::spawn(async move {
        if let Err(err) = runner.run(task, TaskTrigger::Manual, false).await {
            tracing::error!(error = %err, "manual trigger failed");
        }
    });

    Ok(Json(serde_json::json!({"ok": true, "task": req.task})))
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.into(),
        }
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.into(),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        let scrubbed = crate::logger::redact(&format!("{err:#}")).into_owned();
        Self::internal(scrubbed)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(serde_json::json!({"error": self.message})),
        )
            .into_response()
    }
}
