# GUI Performance Baseline

## Capture Procedure

Run:

1. `pnpm ui:lint`
2. `pnpm ui:check`
3. `pnpm ui:test`
4. `pnpm ui:build`

For runtime profile logs, run app in dev mode and inspect `render.profile` events for:

- `tasks.workspace`
- `kanban.workspace`
- `calendar.workspace`

## Current Snapshot (2026-02-23)

- UI build output:
  - JS bundle: `dist/assets/index-BD-PVOTM.js` = 624.98 kB (gzip 187.80 kB)
  - CSS bundle: `dist/assets/index-BGjvgXvG.css` = 11.92 kB (gzip 3.18 kB)
- Contract tests:
  - 5/5 passing (`web/api/schemas.test.ts`)
- Typecheck:
  - pass
- Lint:
  - pass

## Responsiveness Hardening Applied

- Task list virtualization with `@tanstack/react-virtual`.
- Debounced task search input.
- Memoized derived selectors/facets for tasks and kanban.
- Batched state replacement for board-delete task cleanup.
- Selector performance guard test for large dataset:
  - `web/store/selectors.test.ts` (12k task filter budget assertion).

## Remaining Perf Work

- Add explicit benchmark dataset run (>500 tasks).
- Record tab switch render durations with percentile summaries.
- Split large production chunk for improved startup/download cost.
