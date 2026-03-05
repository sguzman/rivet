# Rivet Map Viewer (Martin Tiles) Roadmap

## 0. Goal

- [x] Add a new `Map` tab in Rivet for interactive map viewing.
- [x] Render locally served tiles from Martin (`http://127.0.0.1:3002`).
- [x] Support smooth pan/zoom with stable performance on desktop.
- [x] Keep implementation focused on viewing only (no search/routing/business data yet).

## 1. Scope and Non-Goals

### In Scope

- [x] Map tab shell and navigation integration.
- [x] Tile rendering from Martin sources.
- [x] Basic viewport controls (zoom in/out, reset view).
- [x] Initial viewport centered to US/Mexico coverage.
- [x] Runtime diagnostics for Martin connectivity and tile availability.

### Out of Scope (for this phase)

- [ ] Address/place search.
- [ ] Turn-by-turn routing.
- [ ] Business/POI overlays and discovery.
- [ ] Offline tile download UI.
- [ ] Editing/annotation/drawing tools.

## 2. Source-of-Truth Inputs

- [x] Docker compose at [tmp/docker-compose.yaml](/win/linux/Code/rust/rivet/tmp/docker-compose.yaml) reviewed.
- [x] Martin is exposed at `127.0.0.1:3002` on host.
- [x] Tile files are mounted from `/win/services/martin/tiles`.
- [x] Martin Web UI is enabled (`--webui enable-for-all`) for inspection/debugging.
- [x] Coverage expectation is primarily US + Mexico tiles.

## 3. Architecture Decisions

- [x] Use a WebGL map client suitable for vector/raster tile rendering (latest stable release at implementation time).
- [x] Keep map rendering in frontend (UI) and avoid proxying tile bytes through Tauri backend.
- [ ] Add a thin backend command only for health/diagnostics metadata if needed.
- [x] Prefer source/layer config driven by Martin capabilities endpoint where feasible.
- [x] Enforce strict separation between map view state and future search/routing state.

## 4. UI and Product Behavior

- [x] Add `Map` tab entry alongside Tasks/Kanban/Calendar/Dictionary.
- [x] Provide full-height map canvas with responsive layout.
- [x] Add minimal controls:
- [x] zoom in
- [x] zoom out
- [x] reset-to-default extent (US/Mexico)
- [x] Add loading, empty, and error overlays:
- [x] loading tiles
- [x] Martin unreachable
- [x] no tiles for current zoom/area
- [x] Preserve user viewport in-session when switching tabs.

## 5. Martin Integration Plan

- [x] Implement tile source bootstrap from Martin endpoint.
- [x] Validate reachable base URL (`http://127.0.0.1:3002`).
- [x] Parse/consume available source metadata (tilesets/layers).
- [x] Select sane default source/layer when multiple are present.
- [x] Configure map client for gzip-encoded tile responses.
- [x] Handle source reload gracefully when Martin restarts.

## 6. Performance Plan

- [x] Keep initial load lightweight (defer heavy style/layer setup until map ready).
- [x] Limit concurrent tile requests per zoom interaction where configurable.
- [x] Set conservative default min/max zoom to reduce unnecessary misses.
- [x] Debounce high-frequency UI state updates from map events.
- [x] Add trace spans and timing around:
- [x] map initialization
- [x] first tile render
- [x] source load duration
- [x] tile error rates
- [x] Verify no unbounded in-memory tile caching in app layer.

## 7. Observability and Logging

- [x] Add structured tracing for map lifecycle events in frontend/backed where relevant.
- [x] Emit startup log with effective Martin URL and map feature flags.
- [x] Log user-visible connectivity failures with actionable diagnostics.
- [x] Log tile source metadata summary at debug level.
- [x] Ensure dev mode writes logs to both stdout and file sinks.

## 8. Config and Environment

- [x] Add map config block in `rivet.toml` (or existing config model):
- [x] `enabled`
- [x] `martin_base_url`
- [x] `default_center`
- [x] `default_zoom`
- [x] `min_zoom`
- [x] `max_zoom`
- [x] `hide_when_unavailable`
- [x] Surface effective map config in runtime snapshot diagnostics.
- [x] Keep defaults aligned with local compose (`http://127.0.0.1:3002`).

## 9. Testing and Verification

### Frontend

- [x] Component tests for tab rendering and control interactions.
- [x] Store/state tests for viewport persistence and error states.
- [x] Mocked integration tests for Martin metadata success/failure.

### Backend (if command hooks added)

- [ ] Unit tests for map config parsing/validation.
- [ ] Command tests for health-check payloads and error mapping.

### Manual/E2E

- [ ] Start compose Martin and verify map tiles render in Map tab.
- [ ] Confirm expected behavior when Martin is stopped.
- [ ] Confirm panning outside US/Mexico coverage shows graceful no-data behavior.
- [ ] Confirm pan/zoom remain responsive during sustained interaction.

## 10. Rollout Phases

- [x] Phase 1: map tab scaffold + static base map client wiring.
- [x] Phase 2: dynamic Martin source discovery + robust error handling.
- [x] Phase 3: performance tuning + tracing + test hardening.
- [ ] Phase 4: polish UX copy and finalize docs.

## 11. Documentation Updates

- [x] Add map tab notes to command/config docs where applicable.
- [x] Document local Martin startup expectations and troubleshooting.
- [x] Add quick validation guide (`docker compose up`, open Map tab, inspect logs).

## 12. Completion Criteria

- [x] Map tab exists and renders Martin-backed tiles reliably.
- [x] Pan/zoom UX is smooth under normal local usage.
- [x] Clear user feedback for unavailable server/tiles.
- [x] Logging/tracing is sufficient to diagnose map load problems quickly.
- [x] Build/test checks pass with map feature enabled.
