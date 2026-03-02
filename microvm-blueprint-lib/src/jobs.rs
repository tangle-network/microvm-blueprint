#[derive(Debug, Clone)]
pub enum LifecycleJob {
    Create { vm_id: String },
    Start { vm_id: String },
    Stop { vm_id: String },
    Snapshot { vm_id: String, snapshot_id: String },
    Destroy { vm_id: String },
}
