**Rivet Yew -> React/TypeScript + Tailwind + MUI Migration Roadmap (based on Voltlane pattern)**

## 0. Scope and Success Criteria

- [ ] Define migration target architecture: Rust core + Rust Tauri command
  bridge + React/TS frontend shell.
- [ ] Confirm non-goals for first cut (for example: no redesign of backend
  domain, no command semantic changes unless needed).
- [ ] Define measurable success gates:
- [ ] `cargo tauri dev` runs with React UI.
- [ ] Feature parity with current Yew app (Tasks, Kanban, Calendar, External
  Calendars, Modals, Notifications, Settings).
- [ ] No command regressions in Tauri bridge.
- [ ] Performance equal or better than current UI responsiveness.
- [ ] Logging remains comprehensive (frontend + tauri + core).

## 1. Baseline Audit and Freeze

- [ ] Freeze current Yew behavior as reference.
- [ ] Capture golden interaction list by view:
- [ ] Tasks: list/filter/search/select/add/edit/delete/done/bulk actions.
- [ ] Kanban: boards CRUD, lane drag/drop, card density toggle, filters.
- [ ] Calendar: year/quarter/month/week/day nav, markers, external calendars,
  period tasks.
- [ ] Modals: add task, board create/rename, external calendar add/edit/delete.
- [ ] Settings: due notifications permissions/pre-notify.
- [ ] Capture command inventory from Tauri (`tasks_*`, `external_calendar_*`,
  `window_*`, logging).
- [ ] Capture current config dependencies (`rivet.toml`, `tags.toml`).
- [ ] Record current frontend performance baseline (initial load, tab switch
  latency, large list rendering).

## 2. Target Architecture Definition (Voltlane-Inspired)

- [ ] Adopt Voltlane-style split:
- [ ] `src-tauri` remains Rust host + command bridge.
- [ ] New `ui` frontend becomes Vite + React + TypeScript.
- [ ] Shared DTOs continue through `rivet_gui_shared` and command contracts.
- [ ] Define runtime strategy:
- [ ] Real Tauri runtime uses `@tauri-apps/api/core invoke`.
- [ ] Optional browser/mock mode for UI-only dev (like Voltlane mock invoke
  fallback).
- [ ] Define typed frontend API layer:
- [ ] One `ui/src/api/tauri.ts` command client.
- [ ] Centralized request/response types mapping.
- [ ] Standardized error normalization and tracing for invoke failures.
- [ ] Define frontend state architecture:
- [ ] Central store (Zustand or Redux Toolkit; choose one).
- [ ] Separate slices: tasks, filters, kanban, calendar, modals, settings,
  external calendars, notifications, ui/session.
- [ ] Define folder structure:
- [ ] `ui/src/app` shell and route/view composition.
- [ ] `ui/src/features/*` domain feature modules.
- [ ] `ui/src/components/*` reusable UI primitives.
- [ ] `ui/src/theme/*` MUI theme and design tokens.
- [ ] `ui/src/lib/*` logger, date/time utils, tag color utils, config adapters.

## 3. Toolchain Bootstrapping

- [ ] Add Node workspace tooling at repo root (pnpm workspace style like
  Voltlane).
- [ ] Add `ui/package.json` with:
- [ ] `react`, `react-dom`, `typescript`, `vite`, `@vitejs/plugin-react`,
  `@tauri-apps/api`.
- [ ] Tailwind stack: `tailwindcss`, `postcss`, `autoprefixer`.
- [ ] MUI stack: `@mui/material`, `@mui/icons-material`, `@emotion/react`,
  `@emotion/styled`.
- [ ] Add TS config (`tsconfig.json`, `tsconfig.node.json`), Vite config, strict
  type settings.
- [ ] Update `src-tauri/tauri.conf.json`:
- [ ] `beforeDevCommand` -> Vite dev server.
- [ ] `beforeBuildCommand` -> Vite build.
- [ ] `devUrl` + `frontendDist` paths aligned to new UI output.
- [ ] Keep Trunk/Yew build path available behind temporary fallback until parity
  complete.

## 4. Styling System Plan (Tailwind + MUI)

- [ ] Define style ownership model to prevent conflicts:
- [ ] Tailwind for layout/spacing/utility classes.
- [ ] MUI for component primitives (buttons, dialogs, menus, forms, tables).
- [ ] Create unified design token map from current CSS vars:
- [ ] Colors, semantic states, border radii, spacing scale, shadows, typography.
- [ ] Implement MUI theme from tokens (light/day and dark/night modes).
- [ ] Implement Tailwind config tokens that mirror same palette and spacing.
- [ ] Define theme switching policy:
- [ ] One source of truth for day/night in store.
- [ ] Sync MUI theme mode + Tailwind class strategy.
- [ ] Audit all current UI pieces for required MUI equivalents.

## 5. Command Bridge Hardening Before UI Rewrite

- [ ] Document all Tauri command contracts in one place (input/output/error
  format).
- [ ] Validate command argument naming consistency for TS invoke payloads.
- [ ] Add backend command trace IDs to correlate frontend invokes with Rust
  logs.
- [ ] Ensure long-running commands have timeout/cancellation strategy in
  frontend API layer.
- [ ] Add explicit command health checks (simple invoke smoke set on app
  startup).

## 6. Incremental Migration Strategy (No Big Bang)

- [ ] Introduce React UI shell while keeping Rust backend unchanged.
- [ ] Phase migration by feature modules:
- [ ] Phase A: Global shell (window chrome, tabs, basic layout).
- [ ] Phase B: Tasks workspace.
- [ ] Phase C: Kanban workspace.
- [ ] Phase D: Calendar workspace.
- [ ] Phase E: Settings + notifications + modal ecosystem polish.
- [ ] Keep Yew UI branchable fallback until each phase passes parity checklist.
- [ ] Define feature-flag style cutover (build-time or runtime) during
  transition.

## 7. React App Shell Implementation Plan

- [ ] Build top-level app frame matching current IA:
- [ ] header/window chrome region
- [ ] workspace tabs (`Tasks`, `Kanban`, `Calendar`)
- [ ] main 3-column adaptable layout
- [ ] Port global actions and global keyboard handlers.
- [ ] Port global modal host and z-index layering model.
- [ ] Implement structured frontend logger (`ui/src/lib/logger.ts`) with
  tracing-style event names.

## 8. Tasks Feature Migration Plan

- [ ] Port task list rendering with performant virtualization strategy for large
  datasets.
- [ ] Port filters and facets:
- [ ] project/tag/completion/priority/due/search
- [ ] clear filters behavior parity
- [ ] Port details pane actions:
- [ ] edit/done/delete/bulk actions
- [ ] Port add/edit task modal:
- [ ] title required, description optional
- [ ] tags freeform + picker model
- [ ] kanban board lock behavior when launched from kanban
- [ ] recurrence section and validation
- [ ] Ensure “Saving...” behavior has proper command completion and error
  unwind.
- [ ] Preserve tag chip color rendering by key/value rules.

## 9. Kanban Feature Migration Plan

- [ ] Port board list sidebar:
- [ ] create/rename/delete/select board
- [ ] board color display
- [ ] Port lane rendering from tag schema (`kanban` key values).
- [ ] Port drag-and-drop with robust DnD library and hitbox ergonomics.
- [ ] Port card density toggle.
- [ ] Port kanban-specific filters.
- [ ] Ensure move operation updates both backend and local optimistic state
  safely.
- [ ] Validate tag updates when moving cards across lanes and between boards.

## 10. Calendar Feature Migration Plan

- [ ] Port calendar views:
- [ ] year/quarter/month/week/day
- [ ] Port navigation semantics:
- [ ] year/quarter month click -> month view
- [ ] month week shortcuts -> week view
- [ ] Port markers and shapes:
- [ ] triangle/kanban-board color
- [ ] circle/external calendar color
- [ ] square/unaffiliated gray
- [ ] Port right pane stats + current-period tasks and calendar filtering.
- [ ] Port external calendar source management:
- [ ] add/edit/delete
- [ ] import ICS
- [ ] sync enabled/manual/refresh controls
- [ ] imported ICS constraints (no recurrence/sync where disabled by policy).

## 11. Config Consumption Strategy in React

- [ ] Replace compile-time `include_str!` frontend config assumptions with
  runtime-loaded config payload from Tauri command.
- [ ] Add explicit tauri command(s) for frontend config snapshot:
- [ ] `rivet.toml` effective config
- [ ] tag schema (`tags.toml`) data
- [ ] Build config adapter in TS to normalize defaults.
- [ ] Ensure one canonical source of truth for:
- [ ] timezone
- [ ] calendar policies
- [ ] mode/logging indicators for diagnostics UI.

## 12. Data and Type Safety Strategy

- [ ] Generate/maintain TS types for all command DTOs from Rust shared contracts
  where practical.
- [ ] Add runtime validation for external data boundaries (zod/io-ts) at API
  edge.
- [ ] Enforce strict TS (`noImplicitAny`, `strictNullChecks`, etc.).
- [ ] Add lint + format gates for TS/React/Tailwind.
- [ ] Add API contract tests to detect Rust/TS drift.

## 13. Logging and Observability Plan

- [ ] Frontend event logging:
- [ ] command start/success/error with durations
- [ ] user interaction breadcrumbs for buttons/modal actions
- [ ] Keep backend tracing as-is with dev log file mode.
- [ ] Define shared correlation id propagation:
- [ ] frontend invoke includes request id
- [ ] tauri logs request id
- [ ] Display diagnostics panel for last N command failures in dev mode.

## 14. Performance and Responsiveness Plan

- [ ] Add React profiler pass on heavy views (Tasks/Calendar).
- [ ] Memoize derived selectors for filters and facets.
- [ ] Batch updates to avoid re-render storms.
- [ ] Use virtualized list for large task sets.
- [ ] Debounce expensive query/filter operations.
- [ ] Benchmark before/after against frozen baseline.

## 15. QA and Parity Validation Plan

- [ ] Build parity checklist document per workflow.
- [ ] Add end-to-end smoke suite (Playwright recommended):
- [ ] add/edit/delete task
- [ ] kanban board CRUD + DnD
- [ ] calendar navigation + marker rendering
- [ ] external calendar import/sync modal behaviors
- [ ] theme toggle and persistence
- [ ] Add regression scenarios for previously broken interactions:
- [ ] dead clicks
- [ ] modal lockups
- [ ] save hangs
- [ ] drag instability
- [ ] Run matrix:
- [ ] Linux/Wayland primary
- [ ] dark/light themes
- [ ] large dataset (hundreds of tasks)

## 16. Cutover Plan

- [ ] Enable React UI as default frontend in tauri config.
- [ ] Keep Yew code present but inactive for one stabilization release.
- [ ] Run bugfix hardening sprint (no feature work).
- [ ] Remove Trunk/Yew pipeline only after parity signoff:
- [ ] remove Yew deps and wasm plumbing
- [ ] remove Trunk config + obsolete Rust frontend files
- [ ] update workspace and CI scripts to Node+Vite frontend flow.

## 17. CI/CD Plan

- [ ] Add frontend CI stages:
- [ ] `pnpm install`
- [ ] `pnpm --dir ui run check`
- [ ] `pnpm --dir ui run build`
- [ ] Keep Rust checks:
- [ ] `cargo fmt --check`
- [ ] `cargo clippy ...`
- [ ] `cargo test`
- [ ] Add integrated app build check (`cargo tauri build` smoke in CI when
  feasible).
- [ ] Add artifact retention for frontend build + tauri logs in dev pipelines.

## 18. Documentation Deliverables

- [ ] New architecture doc: Rust core + Tauri bridge + React shell.
- [ ] Frontend contribution guide:
- [ ] component patterns
- [ ] state management
- [ ] Tailwind + MUI usage rules
- [ ] command API usage
- [ ] Migration notes for removed Yew modules.
- [ ] Troubleshooting guide for Wayland/Tauri interaction issues.

## 19. Recommended Execution Order (Practical)

- [ ] Week 1: bootstrap toolchain + shell + typed API client + config command
  path.
- [ ] Week 2: tasks workspace + modals + filters parity.
- [ ] Week 3: kanban workspace + DnD hardening + board management.
- [ ] Week 4: calendar workspace + external calendars + markers.
- [ ] Week 5: polish, e2e, perf tuning, parity closure.
- [ ] Week 6: cutover + deprecate Yew path + docs finalization.

## 20. Risks and Mitigations

- [ ] Risk: contract drift between Rust DTOs and TS.
- [ ] Mitigation: generated/shared typings + API contract tests.
- [ ] Risk: Tailwind/MUI style conflicts.
- [ ] Mitigation: explicit style ownership, token map, lint rules.
- [ ] Risk: interaction regressions during DnD/modal rewrites.
- [ ] Mitigation: dedicated interaction regression suite and scripted QA.
- [ ] Risk: config divergence from runtime assumptions.
- [ ] Mitigation: single `rivet.toml` source + explicit config fetch command.
- [ ] Risk: migration drag from big-bang rewrite.
- [ ] Mitigation: phased cutover with feature-by-feature parity gates.

If you want, next I can turn this into a milestone board with strict “entry/exit criteria” per phase and estimated task counts per module.
