# microvm-blueprint

Minimal runnable workspace blueprint for microVM lifecycle orchestration.

## Workspace

- `microvm-blueprint-lib`: reusable library with lifecycle job definitions, a job runner, query traits, and a mocked provider adapter.
- `microvm-blueprint-bin`: runnable binary that wires the runner and exposes read-only query surfaces.

## Lifecycle jobs (state-changing only)

The library defines `LifecycleJob` variants for mutating operations only:

- `Create`
- `Start`
- `Stop`
- `Snapshot`
- `Destroy`

These are executed through `JobRunner` backed by a provider implementing `VmProvider`.

## Query surfaces (read-only only)

The binary exposes read-only query access through:

- HTTP endpoints:
  - `GET /health`
  - `GET /vms`
  - `GET /vms/{vm_id}`
  - `GET /vms/{vm_id}/snapshots`
- Background query monitor task (periodic VM count logging).

No state-changing operations are exposed as HTTP endpoints in this skeleton.

## Run

```bash
cargo run -p microvm-blueprint-bin
```

Then query:

```bash
curl http://127.0.0.1:3000/vms
curl http://127.0.0.1:3000/vms/demo-vm
curl http://127.0.0.1:3000/vms/demo-vm/snapshots
```

## Validate

```bash
cargo check
```
