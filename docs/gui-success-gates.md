# GUI Migration Success Gates

Status date: 2026-02-23

## Gate Checklist

- [x] `cargo tauri dev` wiring points to Vite frontend (`tauri.conf.json`).
- [x] React shell covers Tasks, Kanban, Calendar, External Calendars, Settings, modals.
- [x] Command bridge has typed API adapter and runtime schema validation.
- [x] Legacy Yew fallback path removed from scripts/CI/build configs after React cutover stabilization.
- [x] Frontend and backend emit correlated logs with `request_id`.
- [x] Frontend quality gates pass locally:
  - `pnpm ui:lint`
  - `pnpm ui:check`
  - `pnpm ui:test`
  - `pnpm ui:build`
- [x] Rust workspace checks/tests pass:
  - `cargo check --workspace`
  - `cargo test --workspace`
- [x] Runtime dev startup probe:
  - `cargo tauri dev --no-watch --no-dev-server-wait` starts Vite and Tauri process with React shell wiring.
- [x] Local Playwright smoke execution stable in this host after frontend logging recursion fix.
- [x] CI smoke matrix covers:
  - Linux + Wayland env startup probe
  - Day and Night themes
  - Small and large task datasets

## Evidence Artifacts

- Command contracts: `docs/tauri-command-contracts.md`
- Architecture: `docs/gui-architecture.md`
- Parity checklist: `docs/gui-parity-checklist.md`
- Performance baseline: `docs/gui-performance-baseline.md`
