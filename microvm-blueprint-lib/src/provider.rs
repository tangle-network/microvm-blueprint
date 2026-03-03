//! Compatibility re-exports for runtime provider traits/adapters.

#[cfg(feature = "firecracker")]
pub use microvm_runtime::{FirecrackerConfig, FirecrackerVmProvider};
pub use microvm_runtime::{InMemoryVmProvider, VmProvider, VmQuery, VmRuntime};
