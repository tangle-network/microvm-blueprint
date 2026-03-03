# Architecture

## Goals

- Keep VM state mutation in explicit lifecycle jobs dispatched by blueprint-sdk's `Router`.
- Keep read-only queries separate from jobs, served as a `BackgroundService`.
- Provide a clean provider trait boundary (`VmProvider`/`VmQuery`) so the in-memory adapter can be swapped for a real hypervisor backend without changing job or query wiring.

## Components

### `microvm-runtime`

- `provider.rs`:
  - `VmProvider` trait — state-changing lifecycle contract.
  - `VmQuery` trait — read-only query contract.
  - `VmRuntime` trait object boundary (`VmProvider + VmQuery`).
- `adapters/in_memory.rs`: deterministic in-memory adapter for development and tests.
- `adapters/firecracker.rs` (feature-gated): Firecracker process lifecycle via unix socket API (create/configure/start/pause/resume/snapshot/destroy).
- `model.rs`: shared read model (`VmView`, `VmStatus`).
- `error.rs`: shared runtime error model (`VmRuntimeError`).

### `microvm-blueprint-lib`

- `jobs.rs`: Async job functions (`create_vm`, `start_vm`, `stop_vm`, `snapshot_vm`, `destroy_vm`) with `TangleArg`/`TangleResult` extractors and `#[debug_job]` macro. Job ID constants (`JOB_CREATE` through `JOB_DESTROY`) map to on-chain Tangle service manager indices.
- `provider.rs`: compatibility re-exports from `microvm-runtime`.
- `query.rs`: `QueryService` implementing `BackgroundService` — spawns an axum HTTP server for read-only query endpoints.
- `model.rs` and `errors.rs`: compatibility re-exports from `microvm-runtime`.
- `lib.rs`: Exports `router()` function that wires all job functions with `TangleLayer`, plus `init_provider`/`vm_provider` for global provider access.

### `microvm-blueprint-bin`

- Loads `BlueprintEnvironment` and connects to Tangle.
- Creates `TangleProducer`/`TangleConsumer` for on-chain job polling and result submission.
- Initializes a VM runtime provider via `init_provider` (in-memory by default; Firecracker feature-gated).
- Runs `BlueprintRunner::builder()` with:
  - `router()` from the lib crate (job dispatch)
  - `QueryService` as a background service (HTTP query endpoints)
  - Tangle producer/consumer (on-chain integration)
  - Graceful shutdown handler

## Flow

1. `TangleProducer` polls for `JobSubmitted` events from the Tangle service manager contract.
2. `BlueprintRunner` dispatches incoming job calls through the `Router`.
3. Each job function decodes ABI arguments via `TangleArg`, calls `VmProvider` methods on the global provider, and returns `TangleResult`.
4. `TangleConsumer` submits ABI-encoded results back to the contract.
5. `QueryService` runs independently, serving read-only HTTP endpoints via `VmQuery`.

## Extending to production

- Add jailer/cgroup/network/vsock orchestration to `FirecrackerVmProvider` for production isolation.
- Persist lifecycle state in a durable store.
- Add authn/authz and observability around query endpoints.
- Add retries/idempotency keys and timeout policies per job type.
- Wire QoS heartbeat service (see `blueprint-qos` crate).
