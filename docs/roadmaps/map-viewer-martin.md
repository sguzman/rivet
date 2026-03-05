# Rivet Map Viewer (Martin Tiles) Roadmap

## 0. Goal

- [ ] Add a new `Map` tab in Rivet for interactive map viewing.
- [ ] Render locally served tiles from Martin (`http://127.0.0.1:3002`).
- [ ] Support smooth pan/zoom with stable performance on desktop.
- [ ] Keep implementation focused on viewing only (no search/routing/business data yet).

## 1. Scope and Non-Goals

### In Scope

- [ ] Map tab shell and navigation integration.
- [ ] Tile rendering from Martin sources.
- [ ] Basic viewport controls (zoom in/out, reset view).
- [ ] Initial viewport centered to US/Mexico coverage.
- [ ] Runtime diagnostics for Martin connectivity and tile availability.

### Out of Scope (for this phase)

- [ ] Address/place search.
- [ ] Turn-by-turn routing.
- [ ] Business/POI overlays and discovery.
- [ ] Offline tile download UI.
- [ ] Editing/annotation/drawing tools.

## 2. Source-of-Truth Inputs

- [ ] Docker compose at [tmp/docker-compose.yaml](/win/linux/Code/rust/rivet/tmp/docker-compose.yaml) reviewed.
- [ ] Martin is exposed at `127.0.0.1:3002` on host.
- [ ] Tile files are mounted from `/win/services/martin/tiles`.
- [ ] Martin Web UI is enabled (`--webui enable-for-all`) for inspection/debugging.
- [ ] Coverage expectation is primarily US + Mexico tiles.

## 3. Architecture Decisions

- [ ] Use a WebGL map client suitable for vector/raster tile rendering (latest stable release at implementation time).
- [ ] Keep map rendering in frontend (UI) and avoid proxying tile bytes through Tauri backend.
- [ ] Add a thin backend command only for health/diagnostics metadata if needed.
- [ ] Prefer source/layer config driven by Martin capabilities endpoint where feasible.
- [ ] Enforce strict separation between map view state and future search/routing state.

## 4. UI and Product Behavior

- [ ] Add `Map` tab entry alongside Tasks/Kanban/Calendar/Dictionary.
- [ ] Provide full-height map canvas with responsive layout.
- [ ] Add minimal controls:
- [ ] zoom in
- [ ] zoom out
- [ ] reset-to-default extent (US/Mexico)
- [ ] Add loading, empty, and error overlays:
- [ ] loading tiles
- [ ] Martin unreachable
- [ ] no tiles for current zoom/area
- [ ] Preserve user viewport in-session when switching tabs.

## 5. Martin Integration Plan

- [ ] Implement tile source bootstrap from Martin endpoint.
- [ ] Validate reachable base URL (`http://127.0.0.1:3002`).
- [ ] Parse/consume available source metadata (tilesets/layers).
- [ ] Select sane default source/layer when multiple are present.
- [ ] Configure map client for gzip-encoded tile responses.
- [ ] Handle source reload gracefully when Martin restarts.

## 6. Performance Plan

- [ ] Keep initial load lightweight (defer heavy style/layer setup until map ready).
- [ ] Limit concurrent tile requests per zoom interaction where configurable.
- [ ] Set conservative default min/max zoom to reduce unnecessary misses.
- [ ] Debounce high-frequency UI state updates from map events.
- [ ] Add trace spans and timing around:
- [ ] map initialization
- [ ] first tile render
- [ ] source load duration
- [ ] tile error rates
- [ ] Verify no unbounded in-memory tile caching in app layer.

## 7. Observability and Logging

- [ ] Add structured tracing for map lifecycle events in frontend/backed where relevant.
- [ ] Emit startup log with effective Martin URL and map feature flags.
- [ ] Log user-visible connectivity failures with actionable diagnostics.
- [ ] Log tile source metadata summary at debug level.
- [ ] Ensure dev mode writes logs to both stdout and file sinks.

## 8. Config and Environment

- [ ] Add map config block in `rivet.toml` (or existing config model):
- [ ] `enabled`
- [ ] `martin_base_url`
- [ ] `default_center`
- [ ] `default_zoom`
- [ ] `min_zoom`
- [ ] `max_zoom`
- [ ] `hide_when_unavailable`
- [ ] Surface effective map config in runtime snapshot diagnostics.
- [ ] Keep defaults aligned with local compose (`http://127.0.0.1:3002`).

## 9. Testing and Verification

### Frontend

- [ ] Component tests for tab rendering and control interactions.
- [ ] Store/state tests for viewport persistence and error states.
- [ ] Mocked integration tests for Martin metadata success/failure.

### Backend (if command hooks added)

- [ ] Unit tests for map config parsing/validation.
- [ ] Command tests for health-check payloads and error mapping.

### Manual/E2E

- [ ] Start compose Martin and verify map tiles render in Map tab.
- [ ] Confirm expected behavior when Martin is stopped.
- [ ] Confirm panning outside US/Mexico coverage shows graceful no-data behavior.
- [ ] Confirm pan/zoom remain responsive during sustained interaction.

## 10. Rollout Phases

- [ ] Phase 1: map tab scaffold + static base map client wiring.
- [ ] Phase 2: dynamic Martin source discovery + robust error handling.
- [ ] Phase 3: performance tuning + tracing + test hardening.
- [ ] Phase 4: polish UX copy and finalize docs.

## 11. Documentation Updates

- [ ] Add map tab notes to command/config docs where applicable.
- [ ] Document local Martin startup expectations and troubleshooting.
- [ ] Add quick validation guide (`docker compose up`, open Map tab, inspect logs).

## 12. Completion Criteria

- [ ] Map tab exists and renders Martin-backed tiles reliably.
- [ ] Pan/zoom UX is smooth under normal local usage.
- [ ] Clear user feedback for unavailable server/tiles.
- [ ] Logging/tracing is sufficient to diagnose map load problems quickly.
- [ ] Build/test checks pass with map feature enabled.
