# GUI Architecture

## Overview

Rivet GUI is a three-layer desktop architecture:

1. Rust domain core (`crates/rivet-core`)
2. Tauri command bridge (`crates/rivet-gui/src-tauri`)
3. React + TypeScript shell (`crates/rivet-gui/ui/web`)

The React shell is intentionally thin and delegates task semantics to Rust commands. Frontend state is local session/cache state, not source-of-truth business logic.

## First-Cut Non-Goals

- Redesigning Rust task/calendar domain behavior.
- Changing command semantics without explicit contract revision.

## Runtime Model

- `@tauri-apps/api/core invoke` is used in desktop runtime.
- UI-only runtime uses a local mock transport for browser dev and smoke tests.
- Runtime transport selection:
  - `VITE_RIVET_UI_RUNTIME_MODE=tauri` forces tauri invoke transport.
  - `VITE_RIVET_UI_RUNTIME_MODE=mock` forces local mock transport.
  - unset or `auto` uses tauri transport only when Tauri internals exist.

## State Ownership

- Canonical persisted task/calendar state: Rust datastore via Tauri commands.
- Frontend state store: Zustand (`useAppStore`) for UI/session concerns:
  - active tab, theme mode
  - filters/facets
  - modal open/close state
  - selected task
  - diagnostics and notification UX state
- Local browser storage only holds UI preferences and mock-mode data.

## Command Boundary

All command contracts are documented in `docs/tauri-command-contracts.md`.

- Frontend validates inbound/outbound payloads via zod schemas in:
  - `crates/rivet-gui/ui/web/api/schemas.ts`
- Frontend injects per-command `request_id`.
- Backend command logs include `request_id` for end-to-end correlation.

## UI Composition

- App shell:
  - `Tasks`
  - `Kanban`
  - `Calendar`
- Shared components under `web/components`.
- Feature modules under `web/features`.
- Utility and adapters under `web/lib`.

## Styling Model

- MUI provides component primitives and theme wiring.
- Tailwind provides layout/utilities.
- Shared design tokens live in:
  - `crates/rivet-gui/ui/web/theme/tokens.ts`
- MUI theme and Tailwind config both consume those tokens to avoid drift.

## Logging and Diagnostics

- Frontend interaction and invoke telemetry logs through `web/lib/logger.ts`.
- Backend tracing writes to file in dev mode (`rivet.toml` mode=`dev`), stderr fallback otherwise.
- Diagnostics panel surfaces recent command failures in dev mode.

## Testing Layers

- Type + lint gates: `pnpm ui:check`, `pnpm ui:lint`
- Contract tests: `pnpm ui:test`
- E2E smoke (mock transport): `pnpm ui:e2e`
- Rust checks/tests remain primary for domain integrity.
