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

- [x] Adopt Voltlane-style split:
- [x] `src-tauri` remains Rust host + command bridge.
- [x] New `ui` frontend becomes Vite + React + TypeScript.
- [x] Shared DTOs continue through `rivet_gui_shared` and command contracts.
- [ ] Define runtime strategy:
- [x] Real Tauri runtime uses `@tauri-apps/api/core invoke`.
- [x] Optional browser/mock mode for UI-only dev (like Voltlane mock invoke
  fallback).
- [ ] Define typed frontend API layer:
- [x] One `ui/src/api/tauri.ts` command client.
- [x] Centralized request/response types mapping.
- [x] Standardized error normalization and tracing for invoke failures.
- [ ] Define frontend state architecture:
- [x] Central store (Zustand or Redux Toolkit; choose one).
- [ ] Separate slices: tasks, filters, kanban, calendar, modals, settings,
  external calendars, notifications, ui/session.
- [ ] Define folder structure:
- [x] `ui/src/app` shell and route/view composition.
- [x] `ui/src/features/*` domain feature modules.
- [x] `ui/src/components/*` reusable UI primitives.
- [x] `ui/src/theme/*` MUI theme and design tokens.
- [x] `ui/src/lib/*` logger, date/time utils, tag color utils, config adapters.

## 3. Toolchain Bootstrapping

- [x] Add Node workspace tooling at repo root (pnpm workspace style like
  Voltlane).
- [x] Add `ui/package.json` with:
- [x] `react`, `react-dom`, `typescript`, `vite`, `@vitejs/plugin-react`,
  `@tauri-apps/api`.
- [x] Tailwind stack: `tailwindcss`, `postcss`, `autoprefixer`.
- [x] MUI stack: `@mui/material`, `@mui/icons-material`, `@emotion/react`,
  `@emotion/styled`.
- [x] Add TS config (`tsconfig.json`, `tsconfig.node.json`), Vite config, strict
  type settings.
- [x] Update `src-tauri/tauri.conf.json`:
- [x] `beforeDevCommand` -> Vite dev server.
- [x] `beforeBuildCommand` -> Vite build.
- [x] `devUrl` + `frontendDist` paths aligned to new UI output.
- [ ] Keep Trunk/Yew build path available behind temporary fallback until parity
  complete.

## 4. Styling System Plan (Tailwind + MUI)

- [x] Define style ownership model to prevent conflicts:
- [x] Tailwind for layout/spacing/utility classes.
- [x] MUI for component primitives (buttons, dialogs, menus, forms, tables).
- [ ] Create unified design token map from current CSS vars:
- [ ] Colors, semantic states, border radii, spacing scale, shadows, typography.
- [x] Implement MUI theme from tokens (light/day and dark/night modes).
- [x] Implement Tailwind config tokens that mirror same palette and spacing.
- [ ] Define theme switching policy:
- [x] One source of truth for day/night in store.
- [x] Sync MUI theme mode + Tailwind class strategy.
- [ ] Audit all current UI pieces for required MUI equivalents.

## 5. Command Bridge Hardening Before UI Rewrite

- [ ] Document all Tauri command contracts in one place (input/output/error
  format).
- [x] Validate command argument naming consistency for TS invoke payloads.
- [x] Add backend command trace IDs to correlate frontend invokes with Rust
  logs.
- [x] Ensure long-running commands have timeout/cancellation strategy in
  frontend API layer.
- [x] Add explicit command health checks (simple invoke smoke set on app
  startup).

## 6. Incremental Migration Strategy (No Big Bang)

- [x] Introduce React UI shell while keeping Rust backend unchanged.
- [ ] Phase migration by feature modules:
- [x] Phase A: Global shell (window chrome, tabs, basic layout).
- [x] Phase B: Tasks workspace.
- [x] Phase C: Kanban workspace.
- [x] Phase D: Calendar workspace.
- [x] Phase E: Settings + notifications + modal ecosystem polish.
- [ ] Keep Yew UI branchable fallback until each phase passes parity checklist.
- [ ] Define feature-flag style cutover (build-time or runtime) during
  transition.

## 7. React App Shell Implementation Plan

- [x] Build top-level app frame matching current IA:
- [x] header/window chrome region
- [x] workspace tabs (`Tasks`, `Kanban`, `Calendar`)
- [x] main 3-column adaptable layout
- [x] Port global actions and global keyboard handlers.
- [x] Port global modal host and z-index layering model.
- [x] Implement structured frontend logger (`ui/src/lib/logger.ts`) with
  tracing-style event names.

## 8. Tasks Feature Migration Plan

- [ ] Port task list rendering with performant virtualization strategy for large
  datasets.
- [x] Port filters and facets:
- [x] project/tag/completion/priority/due/search
- [x] clear filters behavior parity
- [x] Port details pane actions:
- [x] edit/done/delete/bulk actions
- [x] Port add/edit task modal:
- [x] title required, description optional
- [x] tags freeform + picker model
- [x] kanban board lock behavior when launched from kanban
- [x] recurrence section and validation
- [x] Ensure “Saving...” behavior has proper command completion and error
  unwind.
- [x] Preserve tag chip color rendering by key/value rules.

## 9. Kanban Feature Migration Plan

- [x] Port board list sidebar:
- [x] create/rename/delete/select board
- [x] board color display
- [x] Port lane rendering from tag schema (`kanban` key values).
- [x] Port drag-and-drop with robust DnD library and hitbox ergonomics.
- [x] Port card density toggle.
- [x] Port kanban-specific filters.
- [x] Ensure move operation updates both backend and local optimistic state
  safely.
- [ ] Validate tag updates when moving cards across lanes and between boards.

## 10. Calendar Feature Migration Plan

- [x] Port calendar views:
- [x] year/quarter/month/week/day
- [x] Port navigation semantics:
- [x] year/quarter month click -> month view
- [x] month week shortcuts -> week view
- [x] Port markers and shapes:
- [x] triangle/kanban-board color
- [x] circle/external calendar color
- [x] square/unaffiliated gray
- [x] Port right pane stats + current-period tasks and calendar filtering.
- [x] Port external calendar source management:
- [x] add/edit/delete
- [x] import ICS
- [x] sync enabled/manual/refresh controls
- [x] imported ICS constraints (no recurrence/sync where disabled by policy).

## 11. Config Consumption Strategy in React

- [x] Replace compile-time `include_str!` frontend config assumptions with
  runtime-loaded config payload from Tauri command.
- [x] Add explicit tauri command(s) for frontend config snapshot:
- [x] `rivet.toml` effective config
- [x] tag schema (`tags.toml`) data
- [x] Build config adapter in TS to normalize defaults.
- [ ] Ensure one canonical source of truth for:
- [x] timezone
- [x] calendar policies
- [x] mode/logging indicators for diagnostics UI.

## 12. Data and Type Safety Strategy

- [x] Generate/maintain TS types for all command DTOs from Rust shared contracts
  where practical.
- [ ] Add runtime validation for external data boundaries (zod/io-ts) at API
  edge.
- [x] Enforce strict TS (`noImplicitAny`, `strictNullChecks`, etc.).
- [ ] Add lint + format gates for TS/React/Tailwind.
- [ ] Add API contract tests to detect Rust/TS drift.

## 13. Logging and Observability Plan

- [x] Frontend event logging:
- [x] command start/success/error with durations
- [x] user interaction breadcrumbs for buttons/modal actions
- [ ] Keep backend tracing as-is with dev log file mode.
- [ ] Define shared correlation id propagation:
- [x] frontend invoke includes request id
- [x] tauri logs request id
- [x] Display diagnostics panel for last N command failures in dev mode.

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

- [x] Enable React UI as default frontend in tauri config.
- [x] Keep Yew code present but inactive for one stabilization release.
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
