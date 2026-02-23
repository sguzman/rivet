# GUI Migration Success Gates

Status date: 2026-02-23

## Gate Checklist

- [x] `cargo tauri dev` wiring points to Vite frontend (`tauri.conf.json`).
- [x] React shell covers Tasks, Kanban, Calendar, External Calendars, Settings, modals.
- [x] Command bridge has typed API adapter and runtime schema validation.
- [x] Legacy fallback path preserved (Yew index/trunk config + yew tauri override + CI compile check).
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

## Known Gap

- [ ] Local Playwright smoke execution is environment-sensitive in this host (Chromium cannot complete `goto` to local Vite server despite successful `curl` and normal browser behavior).
  - CI includes non-blocking E2E smoke and artifact capture for diagnosis.

## Evidence Artifacts

- Command contracts: `docs/tauri-command-contracts.md`
- Architecture: `docs/gui-architecture.md`
- Parity checklist: `docs/gui-parity-checklist.md`
- Performance baseline: `docs/gui-performance-baseline.md`
