//! VM provider traits and in-memory implementation.
//!
//! [`VmProvider`] defines state-changing operations (used by lifecycle jobs).
//! [`VmQuery`] defines read-only operations (used by query surfaces).
//! [`InMemoryVmProvider`] implements both for development and testing;
//! swap it for a hypervisor-backed adapter in production.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    errors::{BlueprintError, BlueprintResult},
    model::{VmStatus, VmView},
};

/// State-changing operations on microVMs, executed by lifecycle jobs.
pub trait VmProvider: Send + Sync + 'static {
    fn create_vm(&self, vm_id: &str) -> BlueprintResult<()>;
    fn start_vm(&self, vm_id: &str) -> BlueprintResult<()>;
    fn stop_vm(&self, vm_id: &str) -> BlueprintResult<()>;
    fn snapshot_vm(&self, vm_id: &str, snapshot_id: &str) -> BlueprintResult<()>;
    fn destroy_vm(&self, vm_id: &str) -> BlueprintResult<()>;
}

/// Read-only queries against microVM state, used by query surfaces.
pub trait VmQuery: Send + Sync + 'static {
    fn list_vms(&self) -> BlueprintResult<Vec<VmView>>;
    fn get_vm(&self, vm_id: &str) -> BlueprintResult<Option<VmView>>;
    fn list_snapshots(&self, vm_id: &str) -> BlueprintResult<Option<Vec<String>>>;
}

/// In-memory VM provider for development and testing.
///
/// Replace with a hypervisor-backed adapter (e.g. Firecracker, Cloud Hypervisor)
/// for production use.
#[derive(Debug, Clone, Default)]
pub struct InMemoryVmProvider {
    state: Arc<RwLock<HashMap<String, VmRecord>>>,
}

#[derive(Debug, Clone)]
struct VmRecord {
    status: VmStatus,
    snapshots: Vec<String>,
}

impl VmRecord {
    fn view(&self, vm_id: &str) -> VmView {
        VmView {
            vm_id: vm_id.to_owned(),
            status: self.status,
            snapshots: self.snapshots.clone(),
        }
    }
}

impl VmProvider for InMemoryVmProvider {
    fn create_vm(&self, vm_id: &str) -> BlueprintResult<()> {
        let mut state = self.state.write().map_err(|_| BlueprintError::StatePoisoned)?;

        if state.contains_key(vm_id) {
            return Err(BlueprintError::VmAlreadyExists(vm_id.to_owned()));
        }

        state.insert(
            vm_id.to_owned(),
            VmRecord {
                status: VmStatus::Created,
                snapshots: Vec::new(),
            },
        );

        Ok(())
    }

    fn start_vm(&self, vm_id: &str) -> BlueprintResult<()> {
        let mut state = self.state.write().map_err(|_| BlueprintError::StatePoisoned)?;
        let record = state
            .get_mut(vm_id)
            .ok_or_else(|| BlueprintError::VmNotFound(vm_id.to_owned()))?;

        match record.status {
            VmStatus::Created | VmStatus::Stopped => {
                record.status = VmStatus::Running;
                Ok(())
            }
            VmStatus::Running => Err(BlueprintError::InvalidTransition {
                vm_id: vm_id.to_owned(),
                from: "running",
                to: "running",
            }),
            VmStatus::Destroyed => Err(BlueprintError::InvalidTransition {
                vm_id: vm_id.to_owned(),
                from: "destroyed",
                to: "running",
            }),
        }
    }

    fn stop_vm(&self, vm_id: &str) -> BlueprintResult<()> {
        let mut state = self.state.write().map_err(|_| BlueprintError::StatePoisoned)?;
        let record = state
            .get_mut(vm_id)
            .ok_or_else(|| BlueprintError::VmNotFound(vm_id.to_owned()))?;

        match record.status {
            VmStatus::Running => {
                record.status = VmStatus::Stopped;
                Ok(())
            }
            VmStatus::Created => Err(BlueprintError::InvalidTransition {
                vm_id: vm_id.to_owned(),
                from: "created",
                to: "stopped",
            }),
            VmStatus::Stopped => Err(BlueprintError::InvalidTransition {
                vm_id: vm_id.to_owned(),
                from: "stopped",
                to: "stopped",
            }),
            VmStatus::Destroyed => Err(BlueprintError::InvalidTransition {
                vm_id: vm_id.to_owned(),
                from: "destroyed",
                to: "stopped",
            }),
        }
    }

    fn snapshot_vm(&self, vm_id: &str, snapshot_id: &str) -> BlueprintResult<()> {
        let mut state = self.state.write().map_err(|_| BlueprintError::StatePoisoned)?;
        let record = state
            .get_mut(vm_id)
            .ok_or_else(|| BlueprintError::VmNotFound(vm_id.to_owned()))?;

        if record.status == VmStatus::Destroyed {
            return Err(BlueprintError::InvalidTransition {
                vm_id: vm_id.to_owned(),
                from: "destroyed",
                to: "snapshot",
            });
        }

        if record.snapshots.iter().any(|existing| existing == snapshot_id) {
            return Err(BlueprintError::SnapshotAlreadyExists {
                vm_id: vm_id.to_owned(),
                snapshot_id: snapshot_id.to_owned(),
            });
        }

        record.snapshots.push(snapshot_id.to_owned());
        Ok(())
    }

    fn destroy_vm(&self, vm_id: &str) -> BlueprintResult<()> {
        let mut state = self.state.write().map_err(|_| BlueprintError::StatePoisoned)?;
        let record = state
            .get_mut(vm_id)
            .ok_or_else(|| BlueprintError::VmNotFound(vm_id.to_owned()))?;

        if record.status == VmStatus::Destroyed {
            return Err(BlueprintError::InvalidTransition {
                vm_id: vm_id.to_owned(),
                from: "destroyed",
                to: "destroyed",
            });
        }

        record.status = VmStatus::Destroyed;
        Ok(())
    }
}

impl VmQuery for InMemoryVmProvider {
    fn list_vms(&self) -> BlueprintResult<Vec<VmView>> {
        let state = self.state.read().map_err(|_| BlueprintError::StatePoisoned)?;
        let mut views = state
            .iter()
            .map(|(vm_id, record)| record.view(vm_id))
            .collect::<Vec<_>>();

        views.sort_by(|a, b| a.vm_id.cmp(&b.vm_id));
        Ok(views)
    }

    fn get_vm(&self, vm_id: &str) -> BlueprintResult<Option<VmView>> {
        let state = self.state.read().map_err(|_| BlueprintError::StatePoisoned)?;
        Ok(state.get(vm_id).map(|record| record.view(vm_id)))
    }

    fn list_snapshots(&self, vm_id: &str) -> BlueprintResult<Option<Vec<String>>> {
        let state = self.state.read().map_err(|_| BlueprintError::StatePoisoned)?;
        Ok(state.get(vm_id).map(|record| record.snapshots.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_lifecycle() {
        let provider = InMemoryVmProvider::default();

        provider.create_vm("vm-a").expect("create");
        provider.start_vm("vm-a").expect("start");
        provider
            .snapshot_vm("vm-a", "snap-1")
            .expect("snapshot");
        provider.stop_vm("vm-a").expect("stop");
        provider.destroy_vm("vm-a").expect("destroy");

        let vm = provider
            .get_vm("vm-a")
            .expect("query")
            .expect("vm exists");

        assert_eq!(vm.status, VmStatus::Destroyed);
        assert_eq!(vm.snapshots, vec!["snap-1".to_owned()]);
    }

    #[test]
    fn create_duplicate_errors() {
        let provider = InMemoryVmProvider::default();
        provider.create_vm("vm-a").expect("first create");
        let err = provider.create_vm("vm-a").unwrap_err();
        assert!(matches!(err, BlueprintError::VmAlreadyExists(_)));
    }

    #[test]
    fn start_nonexistent_errors() {
        let provider = InMemoryVmProvider::default();
        let err = provider.start_vm("missing").unwrap_err();
        assert!(matches!(err, BlueprintError::VmNotFound(_)));
    }

    #[test]
    fn invalid_transition_errors() {
        let provider = InMemoryVmProvider::default();
        provider.create_vm("vm-a").expect("create");

        // Can't stop a created VM (must start first)
        let err = provider.stop_vm("vm-a").unwrap_err();
        assert!(matches!(err, BlueprintError::InvalidTransition { .. }));
    }

    #[test]
    fn list_vms_sorted() {
        let provider = InMemoryVmProvider::default();
        provider.create_vm("vm-c").expect("create c");
        provider.create_vm("vm-a").expect("create a");
        provider.create_vm("vm-b").expect("create b");

        let vms = provider.list_vms().expect("list");
        let ids: Vec<&str> = vms.iter().map(|v| v.vm_id.as_str()).collect();
        assert_eq!(ids, vec!["vm-a", "vm-b", "vm-c"]);
    }

    #[test]
    fn snapshot_duplicate_errors() {
        let provider = InMemoryVmProvider::default();
        provider.create_vm("vm-a").expect("create");
        provider
            .snapshot_vm("vm-a", "snap-1")
            .expect("first snapshot");
        let err = provider.snapshot_vm("vm-a", "snap-1").unwrap_err();
        assert!(matches!(err, BlueprintError::SnapshotAlreadyExists { .. }));
    }

    #[test]
    fn destroy_idempotency_guard() {
        let provider = InMemoryVmProvider::default();
        provider.create_vm("vm-a").expect("create");
        provider.destroy_vm("vm-a").expect("first destroy");
        let err = provider.destroy_vm("vm-a").unwrap_err();
        assert!(matches!(err, BlueprintError::InvalidTransition { .. }));
    }
}
