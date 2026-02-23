# Frontend Contribution Guide

## Stack

- React + TypeScript + Vite
- MUI for component primitives
- Tailwind for layout/utilities
- Zustand for app state
- zod for API edge validation

## Core Rules

1. Keep business semantics in Rust commands, not ad-hoc frontend logic.
2. Validate all command responses crossing the Tauri boundary.
3. Prefer existing shared components before adding new primitives.
4. Use design tokens from `web/theme/tokens.ts` for colors/shape/typography.
5. Maintain tracing breadcrumbs for user-triggered operations.

## File Organization

- `web/app`: shell + cross-feature orchestration
- `web/features`: feature workspaces and feature-specific view logic
- `web/components`: reusable presentational components
- `web/api`: command transport + schema validation
- `web/lib`: pure helpers/adapters
- `web/store`: centralized UI state

## State Management Patterns

- Read store state via selector hooks.
- Keep expensive derivations memoized.
- Avoid repeated state writes inside loops; batch updates where possible.
- Persist only user preferences/session metadata to browser storage.

## Tailwind + MUI Usage Rules

- Use MUI components for controls/dialogs/forms/tables.
- Use Tailwind for structural layout and spacing.
- Do not hardcode colors in components when tokenized values exist.
- Keep theme-dependent visuals in MUI theme or CSS variables/classes.

## Command API Usage

- Use `web/api/tauri.ts` for all command calls.
- Do not call `invoke` directly from feature components.
- Add/update zod schemas in `web/api/schemas.ts` when contracts change.
- Update `docs/tauri-command-contracts.md` when command I/O changes.

## Local Quality Gates

Run before commit:

1. `pnpm ui:lint`
2. `pnpm ui:check`
3. `pnpm ui:test`
4. `pnpm ui:build`

## E2E Smoke

Run browser-mode smoke suite:

1. `pnpm ui:e2e`

Notes:
- E2E suite forces mock transport (`VITE_RIVET_UI_RUNTIME_MODE=mock`) and does not require Tauri.
