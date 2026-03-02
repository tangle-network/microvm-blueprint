# Architecture

## Goals

- Keep VM state mutation in explicit lifecycle jobs.
- Keep read-only queries separate from jobs.
- Provide a minimal adapter + runner wiring that can be replaced with real provider implementations.

## Components

### `microvm-blueprint-lib`

- `jobs.rs`: `LifecycleJob` enum with create/start/stop/snapshot/destroy operations.
- `provider.rs`:
  - `VmProvider` mutation port (for lifecycle job execution).
  - `VmQuery` read-only query port.
  - `MockVmProvider` in-memory adapter implementing both ports.
- `runner.rs`: `JobRunner` that dispatches lifecycle jobs through `VmProvider`.
- `model.rs`: shared read model (`VmView`, `VmStatus`).
- `errors.rs`: shared error model.

### `microvm-blueprint-bin`

- Starts an in-process queue (`tokio::mpsc`) for lifecycle jobs.
- Spawns a job worker task that executes jobs with `JobRunner`.
- Seeds a demo VM via jobs to prove wiring.
- Exposes read-only HTTP query endpoints via Axum.
- Runs a background query monitor that periodically reads VM state (not a job).

## Flow

1. Producers enqueue `LifecycleJob` entries.
2. Job worker consumes queue and invokes `JobRunner::execute`.
3. `JobRunner` calls `VmProvider` adapter methods.
4. Query endpoints and background monitor use `VmQuery` only.

## Extending to production

- Replace `MockVmProvider` with a real hypervisor/cloud adapter.
- Persist lifecycle jobs in a durable queue.
- Add authn/authz and observability around query endpoints.
- Add retries/idempotency keys and timeout policies per job type.
