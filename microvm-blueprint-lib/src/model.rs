use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VmStatus {
    Created,
    Running,
    Stopped,
    Destroyed,
}

#[derive(Debug, Clone, Serialize)]
pub struct VmView {
    pub vm_id: String,
    pub status: VmStatus,
    pub snapshots: Vec<String>,
}
