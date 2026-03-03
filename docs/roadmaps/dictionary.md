# Rivet Dictionary Roadmap

## 0. Scope and Outcome

- [ ] Add a new `Dictionary` workspace tab to Rivet.
- [ ] Use a SQLite-backed lexical dataset (configured path) as the source of truth.
- [ ] Ship a practical first version:
- [ ] language selector
- [ ] searchable word input
- [ ] definition pane with structured sections
- [ ] Keep implementation read-only against dictionary DB.
- [ ] Preserve Rivet standards:
- [ ] strong tracing coverage
- [ ] typed Tauri command contracts
- [ ] build/test verification gates

## 1. Current Status and Constraints

- [ ] Confirm real dictionary DB path in config.
- [ ] Current local file `wiktionary.sqlite` exists but is empty (0 bytes), with no tables/schema.
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

- [ ] Add config section in `rivet.toml`:
- [ ] `[dictionary]`
- [ ] `enabled = true`
- [ ] `sqlite_path = "..."` (absolute or workspace-relative)
- [ ] `default_language = "en"`
- [ ] `max_results = 100`
- [ ] `search_mode = "prefix"` (future: `fts`, `fuzzy`)
- [ ] Expose effective dictionary config through existing `config_snapshot`.
- [ ] Add startup validation in Tauri backend:
- [ ] path exists
- [ ] file readable
- [ ] schema has required tables/columns
- [ ] Log structured warnings/errors with remediation hints.

## 4. Backend API (Tauri) Phase

- [ ] Add read-only dictionary data module in `src-tauri`:
- [ ] connection lifecycle
- [ ] prepared statements
- [ ] query timeout/error handling
- [ ] Add commands:
- [ ] `dictionary_languages() -> string[]`
- [ ] `dictionary_search({ language, query, limit, cursor? }) -> DictionarySearchResult`
- [ ] `dictionary_entry({ language, lemma_id|word }) -> DictionaryEntry`
- [ ] Add shared DTOs in `rivet_gui_shared`:
- [ ] `DictionarySearchHit`
- [ ] `DictionaryEntry`
- [ ] `DictionarySense`
- [ ] `DictionaryPronunciation`
- [ ] `DictionaryMeta`
- [ ] Add tracing spans/fields:
- [ ] `request_id`
- [ ] `language`
- [ ] query length (not full query at info level)
- [ ] result count
- [ ] DB duration ms
- [ ] Add schema/contract tests for DTO serialization.

## 5. Frontend UI Phase (New Tab)

- [ ] Add `Dictionary` tab in `AppShell`.
- [ ] Create `DictionaryWorkspace.tsx`:
- [ ] left/top controls: language select + search input
- [ ] center list: search hits
- [ ] right/detail pane: pronunciation, POS, senses, extra metadata
- [ ] Add store slice:
- [ ] selected language
- [ ] query text
- [ ] loading/error state
- [ ] results and selected entry
- [ ] Add API client methods in `web/api/tauri.ts` with zod validation.
- [ ] Keyboard UX:
- [ ] `Cmd/Ctrl+5` to open Dictionary tab
- [ ] arrow keys to navigate result list
- [ ] `Enter` to open selected entry
- [ ] Render strategy for entry detail:
- [ ] clear section headers
- [ ] ordered senses
- [ ] compact typography for dense lexical data
- [ ] Explicit empty states:
- [ ] no DB configured
- [ ] no schema match
- [ ] no results

## 6. Search and Relevance Upgrades (Full Feature Track)

- [ ] Phase 1: exact + prefix search with indexed columns.
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

- [ ] Backend tracing events:
- [ ] config load/validation
- [ ] query start/success/failure
- [ ] slow query warning threshold
- [ ] Frontend logs:
- [ ] search start/debounce/submit
- [ ] entry open latency
- [ ] command failure capture in diagnostics panel
- [ ] Data safety:
- [ ] enforce read-only mode
- [ ] cap search result limits
- [ ] sanitize/trim inputs

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
- [ ] Build gates:
- [ ] `cargo build`
- [ ] `cargo check -p rivet_gui_tauri`
- [ ] `pnpm ui:check`
- [ ] `pnpm ui:test`

## 10. Delivery Plan

- [ ] Milestone A (MVP): tab + language select + search + definition view.
- [ ] Milestone B: improved relevance + better lexical section rendering.
- [ ] Milestone C: favorites/history + pronunciation/usage enhancements.
- [ ] Milestone D: optional FTS/fuzzy + translation/cross-link features.

## 11. Open Items to Resolve Before Coding

- [ ] Provide populated SQLite file (or dump) for schema mapping.
- [ ] Confirm expected third section besides pronunciation + definition (likely POS/etymology/usage).
- [ ] Decide default behavior when dictionary config is missing or invalid:
- [ ] hide tab
- [ ] or show disabled tab with setup guidance
- [ ] Decide whether dictionary feature is always-on or behind `[ui.features.dictionary]`.
