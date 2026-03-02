# Contributing

## Branching

- Never commit directly to `main`.
- Branch naming:
  - `feat/<scope>`
  - `fix/<scope>`
  - `chore/<scope>`
  - `docs/<scope>`

## Commit quality

- Use Conventional Commit style (`feat:`, `fix:`, `docs:`, `chore:`).
- Keep commits cohesive and reviewable.
- Do not include generated build artifacts.
- Do not include co-author trailers unless explicitly required.

## PR standards

Each PR should include:

- Problem statement
- Scope boundaries (what is explicitly out of scope)
- Validation evidence (`cargo check`, tests, smoke output)
- Follow-up items if intentionally deferred

## Architecture guardrails

- Lifecycle mutations are jobs only (`Create`, `Start`, `Stop`, `Snapshot`, `Destroy`).
- Read-only operations remain query-only (`/health`, `/vms`, VM detail/snapshots).
- Preserve layer direction from product blueprints through shared runtime into infra.
