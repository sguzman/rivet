# Rivet Dictionary Postgres Migration Roadmap

## 0. Goal

- [x] Migrate dictionary storage/query from SQLite to PostgreSQL.
- [x] Remove SQLite-specific dictionary runtime paths from Rivet.
- [x] Keep the existing dictionary UX and command contracts stable where possible.
- [x] Preserve strong tracing, diagnostics, and build/test gates.

## 1. Source-of-Truth Inputs (from `zimrs`)

Connection defaults discovered in [zimrs/config/wiktionary.toml](/win/linux/Code/rust/rivet/zimrs/config/wiktionary.toml):

- [x] Backend target is Postgres-first (`backend = "postgres"`).
- [x] `host = "127.0.0.1"`
- [x] `port = 5432`
- [x] `user = "admin"`
- [x] `password = "admin"`
- [x] `database = "data"`
- [x] `schema = "dictionary"`
- [x] `sslmode = "disable"`
- [x] Retry policy alignment (`connect_timeout_secs`, `max_connection_retries`, `retry_backoff_ms`).

Reference docs:

- [x] [zimrs/README.md](/win/linux/Code/rust/rivet/zimrs/README.md) Postgres backend behavior reviewed.
- [x] [zimrs/src/db.rs](/win/linux/Code/rust/rivet/zimrs/src/db.rs) schema/table/index strategy reviewed.

## 2. Non-Goals / Explicit Removals

- [x] Do not keep dictionary SQLite runtime fallback in Rivet.
- [x] Remove dictionary SQLite file-path validation and SQLite PRAGMA tuning code.
- [x] Remove dictionary SQLite-specific roadmap/doc wording after migration.

## 3. Config Migration (`rivet.toml` + runtime snapshot)

- [x] Replace `[dictionary].sqlite_path` with Postgres config block.
- [x] Add `[dictionary.postgres]` keys:
- [x] `host`
- [x] `port`
- [x] `user`
- [x] `password`
- [x] `database`
- [x] `schema`
- [x] `sslmode`
- [x] `connect_timeout_secs`
- [x] `max_connection_retries`
- [x] `retry_backoff_ms`
- [x] Keep existing dictionary feature keys where still relevant:
- [x] `enabled`
- [x] `default_language`
- [x] `max_results`
- [x] `search_mode`
- [x] `hide_when_unavailable`
- [x] Expose effective Postgres dictionary config through `config_snapshot`.
- [x] Ensure secrets handling policy is documented (password redaction in logs/snapshots).

## 4. Backend Data Layer Refactor (Tauri)

- [x] Introduce Postgres dictionary repository module.
- [x] Add pooled connection lifecycle (bounded pool, timeout, retry).
- [x] Add startup readiness checks:
- [x] host/port reachable
- [x] database exists
- [x] schema exists
- [x] required tables/columns/indexes present
- [x] Implement query adapters for existing commands:
- [x] `dictionary_languages`
- [x] `dictionary_search`
- [x] `dictionary_entry`
- [x] Preserve DTO compatibility in `rivet_gui_shared`.
- [x] Keep result caps and sanitized inputs.
- [x] Remove SQLite-only helper paths from dictionary command module.

## 5. Query Behavior Parity

- [x] Match current modes: `exact`, `prefix`, `fuzzy`.
- [x] Re-implement optional FTS behavior for Postgres (`tsvector`/GIN-backed path).
- [x] Keep morphology fallback behavior.
- [x] Keep language token resolution behavior (`en -> English`, etc.) where needed.
- [x] Keep warning semantics for fallback paths.

## 6. Observability and Safety

- [x] Add structured tracing for Postgres dictionary operations:
- [x] connection acquisition latency
- [x] query latency
- [x] result counts
- [x] retry attempts
- [x] failures with categorized error codes
- [x] Redact credentials from all logs/errors.
- [x] Add slow-query warnings with thresholds.
- [x] Add startup diagnostics message if dictionary backend is unreachable.

## 7. Frontend Impact

- [x] Keep dictionary UI behavior unchanged unless contract changes require updates.
- [x] Replace SQLite-centric diagnostics text (`db path`) with backend-aware text (host/db/schema).
- [x] Keep search-trigger behavior explicit (Search button only).
- [x] Keep existing empty/error states but rewrite copy for Postgres connectivity issues.

## 8. Tests

### Rust

- [x] Unit tests for Postgres config parsing and validation.
- [x] Integration tests against a Postgres fixture schema (tables + seed rows).
- [x] Contract tests for command payload parity with existing frontend schemas.
- [x] Negative tests:
- [x] bad credentials
- [x] missing schema/table
- [x] timeout/retry exhaustion

### Frontend

- [x] Schema tests updated for Postgres-backed runtime config snapshot.
- [x] Store tests for dictionary bootstrap when backend unavailable.
- [x] E2E smoke still passes (`open tab -> select language -> search -> open entry`).

## 9. Performance and Rollout

- [x] Capture baseline query timings vs current implementation for common terms.
- [x] Tune pool and query plans (indexes) for high-cardinality dictionary data.
- [x] Validate no full table scans for common search paths.
- [x] Add migration notes for operators (required Postgres schema/tables).
- [x] Stage rollout:
- [x] phase 1: direct Postgres cutover implemented (no temporary dual-backend gate)
- [x] phase 2: Postgres is default and only backend path
- [x] phase 3: SQLite dictionary runtime paths removed

## 10. Documentation Updates

- [x] Update [docs/tauri-command-contracts.md](/win/linux/Code/rust/rivet/docs/tauri-command-contracts.md) for Postgres backend notes.
- [x] Add `docs/dictionary-postgres-contract.md` with schema/query assumptions.
- [x] Update [docs/roadmaps/dictionary.md](/win/linux/Code/rust/rivet/docs/roadmaps/dictionary.md) to reference this migration.
- [x] Add troubleshooting section for Postgres connectivity/auth/schema errors.

## 11. Build/Verification Gates

- [x] `cargo fmt`
- [x] `cargo check -p rivet_gui_tauri`
- [x] `cargo test -p rivet_gui_tauri`
- [x] `cargo build`
- [x] `pnpm ui:check`
- [x] `pnpm ui:lint`
- [x] `pnpm ui:test`
- [x] targeted dictionary E2E smoke

## 12. Completion Criteria

- [x] Dictionary commands use Postgres only in production code paths.
- [x] SQLite dictionary runtime code removed.
- [x] Config/docs/tests updated and green.
- [x] User-visible dictionary behavior remains stable (except backend-specific diagnostics wording).
