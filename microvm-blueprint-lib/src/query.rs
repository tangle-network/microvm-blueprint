//! Read-only query service exposed as a blueprint background service.
//!
//! Runs an HTTP server alongside the [`BlueprintRunner`](blueprint_sdk::runner::BlueprintRunner),
//! providing read-only access to VM state. No state-changing operations are exposed here.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router as AxumRouter,
};
use blueprint_sdk::runner::BackgroundService;
use blueprint_sdk::runner::error::RunnerError;
use serde::Serialize;
use tokio::sync::oneshot::{self, Receiver};

use crate::provider::{InMemoryVmProvider, VmQuery};
use crate::vm_provider;

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

/// Background service that exposes read-only query HTTP endpoints.
#[derive(Clone)]
pub struct QueryService {
    addr: SocketAddr,
}

impl QueryService {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

impl BackgroundService for QueryService {
    async fn start(&self) -> Result<Receiver<Result<(), RunnerError>>, RunnerError> {
        let (tx, rx) = oneshot::channel();
        let addr = self.addr;

        tokio::spawn(async move {
            let provider = vm_provider().clone();
            let app = AxumRouter::new()
                .route("/health", get(health))
                .route("/vms", get(list_vms))
                .route("/vms/{vm_id}", get(get_vm))
                .route("/vms/{vm_id}/snapshots", get(list_snapshots))
                .with_state(provider);

            let result = match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => {
                    tracing::info!("query service listening on http://{addr}");
                    axum::serve(listener, app)
                        .await
                        .map_err(|e| RunnerError::Other(Box::new(e)))
                }
                Err(e) => Err(RunnerError::Other(Box::new(e))),
            };

            let _ = tx.send(result);
        });

        Ok(rx)
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn list_vms(State(provider): State<Arc<InMemoryVmProvider>>) -> Response {
    match provider.list_vms() {
        Ok(vms) => Json(vms).into_response(),
        Err(e) => internal_error(e.to_string()),
    }
}

async fn get_vm(
    Path(vm_id): Path<String>,
    State(provider): State<Arc<InMemoryVmProvider>>,
) -> Response {
    match provider.get_vm(&vm_id) {
        Ok(Some(vm)) => Json(vm).into_response(),
        Ok(None) => not_found(format!("vm '{vm_id}' not found")),
        Err(e) => internal_error(e.to_string()),
    }
}

async fn list_snapshots(
    Path(vm_id): Path<String>,
    State(provider): State<Arc<InMemoryVmProvider>>,
) -> Response {
    match provider.list_snapshots(&vm_id) {
        Ok(Some(snapshots)) => Json(snapshots).into_response(),
        Ok(None) => not_found(format!("vm '{vm_id}' not found")),
        Err(e) => internal_error(e.to_string()),
    }
}

fn not_found(message: String) -> Response {
    (StatusCode::NOT_FOUND, Json(ErrorBody { error: message })).into_response()
}

fn internal_error(message: String) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody { error: message }),
    )
        .into_response()
}
