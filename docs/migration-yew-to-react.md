# Migration Notes: Yew -> React

## Intent

Rivet migrated the GUI shell from Yew/WebAssembly to React/TypeScript while preserving Rust command semantics and data contracts.

## What Changed

- Frontend runtime:
  - from Yew app code under `crates/rivet-gui/ui/src`
  - to React app code under `crates/rivet-gui/ui/web`
- Tooling:
  - from Trunk-only pipeline
  - to Vite + pnpm workspace flow
- UI state:
  - centralized in Zustand store
- Command edge:
  - typed API adapter + runtime schema validation

## What Stayed Stable

- Rust core task model and command behavior.
- Tauri command bridge as the contract surface.
- Runtime config source (`rivet.toml`) and tag schema source (`tags.toml`).

## Current Legacy Status

- Yew code remains in-repo for stabilization and test continuity.
- React shell is the active default frontend for Tauri.

## Migration Guardrails

1. No silent command contract changes.
2. Every command payload change must update:
   - Rust command handler
   - TS schema
   - contract docs
3. Regressions in interaction (buttons/modals/drag) require smoke coverage.

## Removal Preconditions for Legacy Yew Path

- Parity checklist completed.
- E2E smoke coverage in place for Tasks/Kanban/Calendar critical flows.
- Stabilization release period complete with no critical regressions.
