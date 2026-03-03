//! MicroVM Blueprint Library
//!
//! Infrastructure-layer blueprint for microVM lifecycle orchestration on Tangle.
//!
//! ## Job Matrix
//!
//! | Job ID | Operation | Description |
//! |--------|-----------|-------------|
//! | 0      | Create    | Provision a new microVM |
//! | 1      | Start     | Start a stopped/created microVM |
//! | 2      | Stop      | Stop a running microVM |
//! | 3      | Snapshot  | Capture microVM state |
//! | 4      | Destroy   | Tear down a microVM |
//!
//! ## Query Surfaces
//!
//! Read-only HTTP endpoints exposed as a background service:
//! - `GET /health`
//! - `GET /vms`
//! - `GET /vms/{vm_id}`
//! - `GET /vms/{vm_id}/snapshots`

pub mod errors;
pub mod jobs;
pub mod model;
pub mod provider;
pub mod query;

pub use errors::{BlueprintError, BlueprintResult};
pub use jobs::{
    JOB_CREATE, JOB_DESTROY, JOB_SNAPSHOT, JOB_START, JOB_STOP, create_vm, destroy_vm,
    snapshot_vm, start_vm, stop_vm,
};
pub use model::{VmStatus, VmView};
pub use provider::{InMemoryVmProvider, VmProvider, VmQuery};
pub use query::QueryService;

use blueprint_sdk::tangle::TangleLayer;
use blueprint_sdk::{Job, Router};
use std::sync::{Arc, OnceLock};

/// Global VM provider instance, initialized by the binary before starting the runner.
static VM_PROVIDER: OnceLock<Arc<InMemoryVmProvider>> = OnceLock::new();

/// Initialize the global VM provider. Must be called once before the runner starts.
///
/// # Panics
///
/// Panics if called more than once.
pub fn init_provider(provider: Arc<InMemoryVmProvider>) {
    VM_PROVIDER
        .set(provider)
        .expect("VM provider already initialized");
}

/// Access the global VM provider.
///
/// # Panics
///
/// Panics if [`init_provider`] has not been called.
pub fn vm_provider() -> &'static Arc<InMemoryVmProvider> {
    VM_PROVIDER
        .get()
        .expect("VM provider not initialized — call init_provider first")
}

/// Build the job router for Tangle EVM dispatch.
///
/// Each lifecycle operation is wired to a job ID with the [`TangleLayer`] for
/// ABI encoding/decoding of on-chain calldata.
#[must_use]
pub fn router() -> Router {
    Router::new()
        .route(JOB_CREATE, create_vm.layer(TangleLayer))
        .route(JOB_START, start_vm.layer(TangleLayer))
        .route(JOB_STOP, stop_vm.layer(TangleLayer))
        .route(JOB_SNAPSHOT, snapshot_vm.layer(TangleLayer))
        .route(JOB_DESTROY, destroy_vm.layer(TangleLayer))
}
