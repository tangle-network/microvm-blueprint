//! Lifecycle job functions for Tangle EVM dispatch.
//!
//! Each function maps to a single on-chain job ID. Arguments are ABI-decoded from
//! Tangle calldata via [`TangleArg`]; results are ABI-encoded back via [`TangleResult`].

use blueprint_sdk::macros::debug_job;
use blueprint_sdk::tangle::extract::{TangleArg, TangleResult};

use crate::errors::BlueprintError;
use crate::provider::VmProvider;
use crate::vm_provider;

/// Create a new microVM.
pub const JOB_CREATE: u8 = 0;

/// Start a stopped or newly created microVM.
pub const JOB_START: u8 = 1;

/// Stop a running microVM.
pub const JOB_STOP: u8 = 2;

/// Capture microVM state as a named snapshot.
pub const JOB_SNAPSHOT: u8 = 3;

/// Tear down a microVM.
pub const JOB_DESTROY: u8 = 4;

/// Create a new microVM with the given identifier.
#[debug_job]
pub async fn create_vm(
    TangleArg((vm_id,)): TangleArg<(String,)>,
) -> Result<TangleResult<bool>, String> {
    vm_provider()
        .create_vm(&vm_id)
        .map_err(|e: BlueprintError| e.to_string())?;
    Ok(TangleResult(true))
}

/// Start a stopped or newly created microVM.
#[debug_job]
pub async fn start_vm(
    TangleArg((vm_id,)): TangleArg<(String,)>,
) -> Result<TangleResult<bool>, String> {
    vm_provider()
        .start_vm(&vm_id)
        .map_err(|e: BlueprintError| e.to_string())?;
    Ok(TangleResult(true))
}

/// Stop a running microVM.
#[debug_job]
pub async fn stop_vm(
    TangleArg((vm_id,)): TangleArg<(String,)>,
) -> Result<TangleResult<bool>, String> {
    vm_provider()
        .stop_vm(&vm_id)
        .map_err(|e: BlueprintError| e.to_string())?;
    Ok(TangleResult(true))
}

/// Capture microVM state as a named snapshot.
#[debug_job]
pub async fn snapshot_vm(
    TangleArg((vm_id, snapshot_id)): TangleArg<(String, String)>,
) -> Result<TangleResult<bool>, String> {
    vm_provider()
        .snapshot_vm(&vm_id, &snapshot_id)
        .map_err(|e: BlueprintError| e.to_string())?;
    Ok(TangleResult(true))
}

/// Tear down a microVM.
#[debug_job]
pub async fn destroy_vm(
    TangleArg((vm_id,)): TangleArg<(String,)>,
) -> Result<TangleResult<bool>, String> {
    vm_provider()
        .destroy_vm(&vm_id)
        .map_err(|e: BlueprintError| e.to_string())?;
    Ok(TangleResult(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_ids_are_unique() {
        let ids = [JOB_CREATE, JOB_START, JOB_STOP, JOB_SNAPSHOT, JOB_DESTROY];
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                assert_ne!(ids[i], ids[j], "Job IDs at positions {i} and {j} collide");
            }
        }
    }

    #[test]
    fn job_ids_are_sequential() {
        assert_eq!(JOB_CREATE, 0);
        assert_eq!(JOB_START, 1);
        assert_eq!(JOB_STOP, 2);
        assert_eq!(JOB_SNAPSHOT, 3);
        assert_eq!(JOB_DESTROY, 4);
    }
}
