# Architecture

## Goals

- Keep VM state mutation in explicit lifecycle jobs dispatched by blueprint-sdk's `Router`.
- Keep read-only queries separate from jobs, served as a `BackgroundService`.
- Provide a clean provider trait boundary (`VmProvider`/`VmQuery`) so the in-memory adapter can be swapped for a real hypervisor backend without changing job or query wiring.

## Components

### `microvm-blueprint-lib`

- `jobs.rs`: Async job functions (`create_vm`, `start_vm`, `stop_vm`, `snapshot_vm`, `destroy_vm`) with `TangleArg`/`TangleResult` extractors and `#[debug_job]` macro. Job ID constants (`JOB_CREATE` through `JOB_DESTROY`) map to on-chain Tangle service manager indices.
- `provider.rs`:
  - `VmProvider` trait — state-changing operations (for lifecycle job execution).
  - `VmQuery` trait — read-only query port.
  - `InMemoryVmProvider` — in-memory adapter implementing both ports for development.
- `query.rs`: `QueryService` implementing `BackgroundService` — spawns an axum HTTP server for read-only query endpoints.
- `model.rs`: Shared read model (`VmView`, `VmStatus`).
- `errors.rs`: Shared error model (`BlueprintError`).
- `lib.rs`: Exports `router()` function that wires all job functions with `TangleLayer`, plus `init_provider`/`vm_provider` for global provider access.

### `microvm-blueprint-bin`

- Loads `BlueprintEnvironment` and connects to Tangle.
- Creates `TangleProducer`/`TangleConsumer` for on-chain job polling and result submission.
- Initializes the in-memory VM provider via `init_provider`.
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

- Replace `InMemoryVmProvider` with a real hypervisor adapter (Firecracker, Cloud Hypervisor).
- Persist lifecycle state in a durable store.
- Add authn/authz and observability around query endpoints.
- Add retries/idempotency keys and timeout policies per job type.
- Wire QoS heartbeat service (see `blueprint-qos` crate).
