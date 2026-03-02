use thiserror::Error;

pub type BlueprintResult<T> = Result<T, BlueprintError>;

#[derive(Debug, Error)]
pub enum BlueprintError {
    #[error("vm '{0}' already exists")]
    VmAlreadyExists(String),
    #[error("vm '{0}' not found")]
    VmNotFound(String),
    #[error("invalid vm transition for '{vm_id}': {from} -> {to}")]
    InvalidTransition {
        vm_id: String,
        from: &'static str,
        to: &'static str,
    },
    #[error("snapshot '{snapshot_id}' already exists for vm '{vm_id}'")]
    SnapshotAlreadyExists { vm_id: String, snapshot_id: String },
    #[error("provider state lock poisoned")]
    StatePoisoned,
}
