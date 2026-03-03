![Tangle Network Banner](https://raw.githubusercontent.com/tangle-network/tangle/refs/heads/main/assets/Tangle%20%20Banner.png)

# microvm-blueprint

[![Discord](https://img.shields.io/badge/Discord-Join%20Chat-7289da?logo=discord&logoColor=white)](https://discord.gg/cv8EfJu3Tn)
[![Twitter](https://img.shields.io/twitter/follow/tangle_network?style=social)](https://twitter.com/tangle_network)

Infrastructure-layer blueprint for microVM lifecycle orchestration on Tangle, built with [blueprint-sdk](https://github.com/tangle-network/blueprint).

## Workspace

- `microvm-blueprint-lib`: reusable library with lifecycle job functions, job router, query traits, and an in-memory provider adapter.
- `microvm-blueprint-bin`: runnable binary wiring the `BlueprintRunner` with Tangle producer/consumer and a read-only query background service.

## Lifecycle jobs (state-changing only)

On-chain job functions dispatched through blueprint-sdk's `Router` with `TangleLayer`:

| Job ID | Function | Description |
|--------|----------|-------------|
| 0 | `create_vm` | Provision a new microVM |
| 1 | `start_vm` | Start a stopped/created microVM |
| 2 | `stop_vm` | Stop a running microVM |
| 3 | `snapshot_vm` | Capture microVM state |
| 4 | `destroy_vm` | Tear down a microVM |

Job arguments are ABI-decoded from Tangle calldata via `TangleArg`; results are ABI-encoded back via `TangleResult`.

## Query surfaces (read-only only)

Read-only HTTP endpoints run as a `BackgroundService` alongside the `BlueprintRunner`:

- `GET /health`
- `GET /vms`
- `GET /vms/{vm_id}`
- `GET /vms/{vm_id}/snapshots`

No state-changing operations are exposed as HTTP endpoints.

## Run

```bash
cargo run -p microvm-blueprint-bin
```

The binary requires a Tangle environment configuration (see `BlueprintEnvironment::load()`).

## Validate

```bash
cargo check
cargo test --all
```

## Incremental engineering workflow

- Main branch remains stable; all feature work lands via short-lived branches.
- Keep job/query boundaries strict: state changes in jobs, reads via query surfaces.
- Every change should include tests or explicit rationale when tests are not yet applicable.
- Prefer small, reviewable PRs with one architectural concern each.

See [CONTRIBUTING.md](CONTRIBUTING.md) for branch, commit, and PR conventions.

## References

This blueprint's structure and wiring patterns were derived from:

| Source | What was referenced |
|--------|---------------------|
| [`blueprint`](https://github.com/tangle-network/blueprint) SDK `examples/incredible-squaring/` | Crate layout (`-lib`/`-bin`), `Router` + `TangleLayer` wiring, `#[debug_job]` macro, `TangleArg`/`TangleResult` extractors, `BackgroundService` trait, `BlueprintRunner::builder()` pattern |
| [`ai-agent-sandbox-blueprint`](https://github.com/user/ai-agent-sandbox-blueprint) | Job ID constants as `u8`, `router()` export pattern, background HTTP API alongside runner, `BlueprintEnvironment::load()` / `TangleProducer` / `TangleConsumer` wiring |
| [`ai-trading-blueprints`](https://github.com/user/ai-trading-blueprints) | Multi-crate workspace conventions, job function signatures with `Result<TangleResult<T>, String>` error pattern, provider trait separation |

Canonical file paths inspected:
- `/home/drew/code/blueprint/examples/incredible-squaring/incredible-squaring-lib/src/lib.rs`
- `/home/drew/code/blueprint/examples/incredible-squaring/incredible-squaring-bin/src/main.rs`
- `/home/drew/code/ai-agent-sandbox-blueprint/ai-agent-sandbox-blueprint-lib/src/lib.rs`
- `/home/drew/code/ai-agent-sandbox-blueprint/ai-agent-sandbox-blueprint-bin/src/main.rs`
- `/home/drew/code/ai-trading-blueprints/trading-blueprint-lib/src/lib.rs`
- `/home/drew/code/ai-trading-blueprints/trading-blueprint-bin/src/main.rs`
