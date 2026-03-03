# Rivet Dictionary Roadmap

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
- [ ] Blocked unknowns until real DB is available:
- [ ] table names
- [ ] field names/types
- [ ] indexing strategy
- [ ] relation between language, lemma, pronunciation, and senses

## 2. Data-Contract Discovery (First Required Milestone)

- [ ] Run schema audit on populated DB:
- [ ] `.tables`
- [ ] `.schema`
- [ ] `PRAGMA table_info(...)`
- [ ] `PRAGMA index_list(...)`
- [ ] sample records for 10 words across 3 languages
- [ ] Identify canonical entry sections to render:
- [ ] pronunciation
- [ ] part of speech
- [ ] etymology / usage / examples (exact fields TBD from schema)
- [ ] definitions/senses
- [ ] Write `docs/dictionary-data-contract.md` with:
- [ ] exact SQL queries
- [ ] nullable field behavior
- [ ] search behavior (prefix/exact/fuzzy)
- [ ] performance baseline queries

## 3. Config and Runtime Wiring

- [x] Add config section in `rivet.toml`:
- [x] `[dictionary]`
- [x] `enabled = true`
- [x] `sqlite_path = "..."` (absolute or workspace-relative)
- [x] `default_language = "en"`
- [x] `max_results = 100`
- [ ] `search_mode = "prefix"` (future: `fts`, `fuzzy`)
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
- [ ] query timeout/error handling
- [x] Add commands:
- [x] `dictionary_languages() -> string[]`
- [x] `dictionary_search({ language, query, limit, cursor? }) -> DictionarySearchResult`
- [x] `dictionary_entry({ language, lemma_id|word }) -> DictionaryEntry`
- [x] Add shared DTOs in `rivet_gui_shared`:
- [x] `DictionarySearchHit`
- [x] `DictionaryEntry`
- [ ] `DictionarySense`
- [ ] `DictionaryPronunciation`
- [ ] `DictionaryMeta`
- [x] Add tracing spans/fields:
- [x] `request_id`
- [x] `language`
- [x] query length (not full query at info level)
- [x] result count
- [x] DB duration ms
- [ ] Add schema/contract tests for DTO serialization.

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
- [ ] Keyboard UX:
- [x] `Cmd/Ctrl+5` to open Dictionary tab
- [ ] arrow keys to navigate result list
- [ ] `Enter` to open selected entry
- [x] Render strategy for entry detail:
- [x] clear section headers
- [ ] ordered senses
- [x] compact typography for dense lexical data
- [x] Explicit empty states:
- [x] no DB configured
- [x] no schema match
- [x] no results

## 6. Search and Relevance Upgrades (Full Feature Track)

- [x] Phase 1: exact + prefix search with indexed columns.
- [ ] Phase 2: typo tolerance/fuzzy ranking (Levenshtein/trigram depending on schema support).
- [ ] Phase 3: morphology helpers:
- [ ] stemming/lemmatization fallback
- [ ] inflected-form redirect to lemma
- [ ] Phase 4: multi-language cross-links (translations, related terms).
- [ ] Optional phase: SQLite FTS virtual table integration if dataset supports it.

## 7. Advanced Dictionary Features

- [ ] Pronunciation enrichments:
- [ ] IPA display
- [ ] syllabification
- [ ] optional audio link/button (if dataset has URLs)
- [ ] Usage enrichments:
- [ ] examples
- [ ] synonyms/antonyms
- [ ] domain/register labels
- [ ] User productivity extras:
- [ ] search history
- [ ] pinned/favorite entries
- [ ] copy definition/IPA actions
- [ ] open definition in split-pane with Tasks tab (future)

## 8. Logging, Diagnostics, and Safety

- [x] Backend tracing events:
- [x] config load/validation
- [x] query start/success/failure
- [ ] slow query warning threshold
- [x] Frontend logs:
- [x] search start/debounce/submit
- [x] entry open latency
- [x] command failure capture in diagnostics panel
- [x] Data safety:
- [x] enforce read-only mode
- [x] cap search result limits
- [x] sanitize/trim inputs

## 9. Testing and Verification

- [ ] Rust:
- [ ] unit tests for query adapters and DTO mapping
- [ ] integration tests against fixture DB
- [ ] command contract tests
- [ ] Frontend:
- [ ] selector/filter behavior tests
- [ ] workspace rendering tests
- [ ] keyboard navigation tests
- [ ] E2E smoke:
- [ ] open tab
- [ ] select language
- [ ] search word
- [ ] open entry
- [x] Build gates:
- [x] `cargo build`
- [x] `cargo check -p rivet_gui_tauri`
- [x] `pnpm ui:check`
- [x] `pnpm ui:test`

## 10. Delivery Plan

- [x] Milestone A (MVP): tab + language select + search + definition view.
- [ ] Milestone B: improved relevance + better lexical section rendering.
- [ ] Milestone C: favorites/history + pronunciation/usage enhancements.
- [ ] Milestone D: optional FTS/fuzzy + translation/cross-link features.

## 11. Open Items to Resolve Before Coding

- [ ] Provide populated SQLite file (or dump) for schema mapping.
- [ ] Confirm expected third section besides pronunciation + definition (likely POS/etymology/usage).
- [ ] Decide default behavior when dictionary config is missing or invalid:
- [ ] hide tab
- [ ] or show disabled tab with setup guidance
- [x] Decide whether dictionary feature is always-on or behind `[ui.features.dictionary]`.
