# Rivet Dictionary Roadmap

> Status: superseded for backend storage by [dictionary-postgres-migration.md](/win/linux/Code/rust/rivet/docs/roadmaps/dictionary-postgres-migration.md).  
> This roadmap tracks feature surface; storage/backend migration now follows the Postgres roadmap.

## 0. Scope and Outcome

- [x] Add a new `Dictionary` workspace tab to Rivet.
- [x] Use a SQLite-backed lexical dataset (configured path) as the source of truth.
- [x] Ship a practical first version:
- [x] language selector
- [x] searchable word input
- [x] definition pane with structured sections
- [x] Keep implementation read-only against dictionary DB.
- [x] Preserve Rivet standards:
- [x] strong tracing coverage
- [x] typed Tauri command contracts
- [x] build/test verification gates

## 1. Current Status and Constraints

- [x] Confirm real dictionary DB path in config.
- [x] Current local file `wiktionary.sqlite` exists but is empty (0 bytes), with no tables/schema.
- [x] Blocked unknowns until real DB is available:
- [x] table names
- [x] field names/types
- [x] indexing strategy
- [x] relation between language, lemma, pronunciation, and senses

## 2. Data-Contract Discovery (First Required Milestone)

- [x] Run schema audit on populated DB:
- [x] `.tables`
- [x] `.schema`
- [x] `PRAGMA table_info(...)`
- [x] `PRAGMA index_list(...)`
- [x] sample records for 10 words across 3 languages
- [x] Identify canonical entry sections to render:
- [x] pronunciation
- [x] part of speech
- [x] etymology / usage / examples (exact fields TBD from schema)
- [x] definitions/senses
- [x] Write `docs/dictionary-data-contract.md` with:
- [x] exact SQL queries
- [x] nullable field behavior
- [x] search behavior (prefix/exact/fuzzy)
- [x] performance baseline queries

## 3. Config and Runtime Wiring

- [x] Add config section in `rivet.toml`:
- [x] `[dictionary]`
- [x] `enabled = true`
- [x] `sqlite_path = "..."` (absolute or workspace-relative)
- [x] `default_language = "en"`
- [x] `max_results = 100`
- [x] `search_mode = "prefix"` (future: `fts`, `fuzzy`)
- [x] Expose effective dictionary config through existing `config_snapshot`.
- [x] Add startup validation in Tauri backend:
- [x] path exists
- [x] file readable
- [x] schema has required tables/columns
- [x] Log structured warnings/errors with remediation hints.

## 4. Backend API (Tauri) Phase

- [x] Add read-only dictionary data module in `src-tauri`:
- [x] connection lifecycle
- [x] prepared statements
- [x] query timeout/error handling
- [x] Add commands:
- [x] `dictionary_languages() -> string[]`
- [x] `dictionary_search({ language, query, limit, cursor? }) -> DictionarySearchResult`
- [x] `dictionary_entry({ language, lemma_id|word }) -> DictionaryEntry`
- [x] Add shared DTOs in `rivet_gui_shared`:
- [x] `DictionarySearchHit`
- [x] `DictionaryEntry`
- [x] `DictionarySense`
- [x] `DictionaryPronunciation`
- [x] `DictionaryMeta`
- [x] Add tracing spans/fields:
- [x] `request_id`
- [x] `language`
- [x] query length (not full query at info level)
- [x] result count
- [x] DB duration ms
- [x] Add schema/contract tests for DTO serialization.

## 5. Frontend UI Phase (New Tab)

- [x] Add `Dictionary` tab in `AppShell`.
- [x] Create `DictionaryWorkspace.tsx`:
- [x] left/top controls: language select + search input
- [x] center list: search hits
- [x] right/detail pane: pronunciation, POS, senses, extra metadata
- [x] Add store slice:
- [x] selected language
- [x] query text
- [x] loading/error state
- [x] results and selected entry
- [x] Add API client methods in `web/api/tauri.ts` with zod validation.
- [x] Keyboard UX:
- [x] `Cmd/Ctrl+5` to open Dictionary tab
- [x] arrow keys to navigate result list
- [x] `Enter` to open selected entry
- [x] Render strategy for entry detail:
- [x] clear section headers
- [x] ordered senses
- [x] compact typography for dense lexical data
- [x] Explicit empty states:
- [x] no DB configured
- [x] no schema match
- [x] no results

## 6. Search and Relevance Upgrades (Full Feature Track)

- [x] Phase 1: exact + prefix search with indexed columns.
- [x] Phase 2: typo tolerance/fuzzy ranking (Levenshtein/trigram depending on schema support).
- [x] Phase 3: morphology helpers:
- [x] stemming/lemmatization fallback
- [x] inflected-form redirect to lemma
- [x] Phase 4: multi-language cross-links (translations, related terms).
- [x] Optional phase: SQLite FTS virtual table integration if dataset supports it.

## 7. Advanced Dictionary Features

- [x] Pronunciation enrichments:
- [x] IPA display
- [x] syllabification
- [x] optional audio link/button (if dataset has URLs)
- [x] Usage enrichments:
- [x] examples
- [x] synonyms/antonyms
- [x] domain/register labels
- [x] User productivity extras:
- [x] search history
- [x] pinned/favorite entries
- [x] copy definition/IPA actions
- [x] open definition in split-pane with Tasks tab (future)

## 8. Logging, Diagnostics, and Safety

- [x] Backend tracing events:
- [x] config load/validation
- [x] query start/success/failure
- [x] slow query warning threshold
- [x] Frontend logs:
- [x] search start/debounce/submit
- [x] entry open latency
- [x] command failure capture in diagnostics panel
- [x] Data safety:
- [x] enforce read-only mode
- [x] cap search result limits
- [x] sanitize/trim inputs

## 9. Testing and Verification

- [x] Rust:
- [x] unit tests for query adapters and DTO mapping
- [x] integration tests against fixture DB
- [x] command contract tests
- [x] Frontend:
- [x] selector/filter behavior tests
- [x] workspace rendering tests
- [x] keyboard navigation tests
- [x] E2E smoke:
- [x] open tab
- [x] select language
- [x] search word
- [x] open entry
- [x] Build gates:
- [x] `cargo build`
- [x] `cargo check -p rivet_gui_tauri`
- [x] `pnpm ui:check`
- [x] `pnpm ui:test`

## 10. Delivery Plan

- [x] Milestone A (MVP): tab + language select + search + definition view.
- [x] Milestone B: improved relevance + better lexical section rendering.
- [x] Milestone C: favorites/history + pronunciation/usage enhancements.
- [x] Milestone D: optional FTS/fuzzy + translation/cross-link features.

## 11. Open Items to Resolve Before Coding

- [x] Provide populated SQLite file (or dump) for schema mapping.
- [x] Confirm expected third section besides pronunciation + definition (likely POS/etymology/usage).
- [x] Decide default behavior when dictionary config is missing or invalid:
- [x] hide tab
- [x] or show disabled tab with setup guidance
- [x] Decide whether dictionary feature is always-on or behind `[ui.features.dictionary]`.
