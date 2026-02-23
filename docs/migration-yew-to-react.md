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

- Legacy Yew/Trunk frontend path has been removed from:
  - workspace members
  - CI checks
  - frontend scripts/configs
  - obsolete Rust/WASM frontend source tree
- React shell is the only active frontend for Tauri.

## Migration Guardrails

1. No silent command contract changes.
2. Every command payload change must update:
   - Rust command handler
   - TS schema
   - contract docs
3. Regressions in interaction (buttons/modals/drag) require smoke coverage.

## Cutover Preconditions (Applied)

- Parity checklist completed.
- E2E smoke coverage in place for Tasks/Kanban/Calendar critical flows.
- Large-dataset and theme matrix runs added to CI.
