//! Lifecycle job functions for Tangle EVM dispatch.
//!
//! Each function maps to a single on-chain job ID. Arguments are ABI-decoded from
//! Tangle calldata via [`TangleArg`]; results are ABI-encoded back via [`TangleResult`].

use blueprint_sdk::macros::debug_job;
use blueprint_sdk::tangle::extract::{TangleArg, TangleResult};

use crate::errors::BlueprintError;
use crate::provider::VmProvider;
use crate::vm_provider;

/// Maximum allowed byte length for identifiers (VM IDs, snapshot IDs).
///
/// Bounds allocations from untrusted Tangle calldata.
const MAX_ID_LEN: usize = 256;

/// Validate that an identifier is non-empty and within length limits.
fn validate_id(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    if value.len() > MAX_ID_LEN {
        return Err(format!(
            "{label} exceeds maximum length of {MAX_ID_LEN} bytes"
        ));
    }
    Ok(())
}

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
    validate_id(&vm_id, "vm_id")?;
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
    validate_id(&vm_id, "vm_id")?;
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
    validate_id(&vm_id, "vm_id")?;
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
    validate_id(&vm_id, "vm_id")?;
    validate_id(&snapshot_id, "snapshot_id")?;
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
    validate_id(&vm_id, "vm_id")?;
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

    #[test]
    fn validate_id_rejects_empty() {
        let err = validate_id("", "vm_id").unwrap_err();
        assert!(err.contains("must not be empty"), "got: {err}");
    }

    #[test]
    fn validate_id_rejects_overlength() {
        let long = "a".repeat(MAX_ID_LEN + 1);
        let err = validate_id(&long, "vm_id").unwrap_err();
        assert!(err.contains("exceeds maximum length"), "got: {err}");
    }

    #[test]
    fn validate_id_accepts_max_length() {
        let at_limit = "a".repeat(MAX_ID_LEN);
        assert!(validate_id(&at_limit, "vm_id").is_ok());
    }

    #[test]
    fn validate_id_accepts_normal() {
        assert!(validate_id("my-vm-01", "vm_id").is_ok());
    }
}
