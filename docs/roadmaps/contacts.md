**Contacts Feature Roadmap (Rivet GUI, No CLI Surface)**

**1. Product Scope**

- [x] Add a new `Contacts` tab to the existing app shell, alongside
   Tasks/Kanban/Calendar.
- [x] Primary layout requirement: left area for search + browsing contacts, right
   area for adding a new contact.
- [x] Keep first release local-first and offline-first, with architecture ready for
   online/contact-source aggregation later.
- [x] Exclude all command-line behavior from scope.

**2. Feature Set Adapted From GNOME Contacts**

- [x] Include now: contact list, search, create, edit, delete, multiple
   phones/emails, typed fields, optional expanded fields, duplicate-aware
   behavior.
- [x] Include now: selection mode for multi-delete and future merge/link.
- [x] Include now: quick actions to launch `mailto:` and `tel:` (where platform
   supports).
- [ ] Include later: source aggregation (local + external).
- [x] Include now: contact linking/unlinking, richer profile metadata, avatars,
   birthday UX baseline.
- [x] Exclude initially: account setup flows, platform account provisioning,
   advanced sync conflict UIs.

**3. UX Structure**

- [x] Left pane: search box, filters, result list, optional selection mode toggle,
   bulk actions bar.
- [x] Right pane: “Add Contact” form by default, with fast create workflow and
   validation.
- [x] Right pane behavior after selection: switch to “Contact Details/Edit” view
   while preserving an “Add New” button.
- [x] Empty states: no contacts yet, no search matches, validation errors,
   loading/failure states.
- [x] Keyboard support: arrow selection, Enter to open, `Cmd/Ctrl+N` add contact,
   `Delete` when in selection mode.

**4. Contact Data Model (V1)**

- [x] `Contact`: `id`, `display_name`, `given_name`, `family_name`, `nickname`,
   `notes`, `created_at`, `updated_at`.
- [x] `Phones[]`: `value`, `type` (`mobile/home/work/other`), `is_primary`.
- [x] `Emails[]`: `value`, `type`, `is_primary`.
- [x] `Websites[]`, `Birthday` (optional), `Organization` (optional), `Title`
   (optional).
- [x] `Addresses[]` deferred to V1.1 unless needed immediately.
- [x] `Source metadata`: `source_id`, `source_kind` (start with `local`),
   `remote_id` nullable.
- [x] `Merged identity hooks` (for future linking): `link_group_id` nullable.

**5. Backend Architecture (Tauri + Rust)**

- [x] Add contact domain/state handling in GUI backend first, parallel to existing
   task commands.
- [x] Persist contacts in dedicated storage, not task records.
- [x] Recommended storage shape: `contacts.data` JSONL (and optional
   `contacts_deleted.data` if soft delete is needed).
- [x] Build normalized in-memory indexes for search (name/email/phone tokens) to
   avoid full scans on every keypress.
- [x] Add DTOs in `rivet-gui-shared` for strict frontend/backend contract typing.
- [x] Keep design ready for eventual migration into `rivet-core` domain if
   cross-surface reuse is needed later.

**6. Tauri Command Contract (V1)**

- [x] `contacts_list(args)` with `query`, `limit`, `cursor`, optional `source`,
   optional `updated_after`.
- [x] `contact_add(args)` with full create payload and server-side validation.
- [x] `contact_update(args)` patch-style updates.
- [x] `contact_delete(args)` single delete.
- [x] `contacts_delete_bulk(args)` bulk delete.
- [x] `contacts_dedupe_preview(args)` optional early dedupe utility.
- [x] `contact_open_action(args)` for `mailto:` / `tel:` launch abstraction.
- [x] Add command correlation + error surfacing exactly like existing command
   telemetry patterns.

**7. Frontend State Design (Zustand)**

- [x] Create a dedicated `contacts` slice, separate from task/kanban/calendar
   concerns.
- [x] Store only UI/session state in Zustand, keep canonical data from backend
   responses.
- [x] State keys: `contacts`, `query`, `selectedContactId`, `selectionIds`,
   `loading`, `error`, `formDraft`, `dirty`.
- [x] Derived selectors must be memoized and cheap, especially for search results
   and facet counts.
- [x] Avoid full-list recomputation on every keystroke by using indexed backend
   query + debounced input.
- [x] Preserve current tab performance discipline: avoid high-frequency bridge
   logging for every input event.

**8. Search and Performance Plan**

- [x] Search behavior: incremental search over name/email/phone with
   case/diacritic-insensitive matching.
- [x] Debounce input (150–250ms) and cancel stale in-flight requests.
- [x] Support pagination or windowing for large sets (start with 200 visible rows
   max + “load more”).
- [x] Virtualize list rendering if contact counts exceed threshold.
- [x] Cache recent query results by key for quick tab revisits.
- [x] Add performance budgets: first result <100ms on warm cache, tab switch <120ms
   target.

**9. Validation and Data Quality**

- [x] Require at least one of: display name, email, or phone.
- [x] Email format validation (strict enough to catch obvious errors, not
   RFC-overkill UX).
- [x] Phone normalization for search but preserve original display formatting.
- [x] Prevent accidental duplicates with non-blocking warning (“possible duplicate
   exists”).
- [x] Enforce safe field limits to prevent runaway payloads.

**10. Milestone Plan**

- [x] **M0 (Foundation)**: DTOs, storage files, command skeleton, tab shell
   integration, empty-state UI.
- [x] **M1 (Core Usable)**: add/list/search/edit/delete single contact, right-pane
   add form, detail panel.
- [x] **M2 (Bulk + Quality)**: selection mode, bulk delete, dedupe warnings,
   keyboard shortcuts, polish.
- [x] **M3 (Parity Enhancements)**: typed fields expansion, communication launch
   actions, richer profile fields.
- [x] **M4 (Advanced Optional)**: contact linking/unlinking and source-aware
   architecture groundwork.
- [ ] **M5 (Future Expansion)**: online source sync/aggregation and
   conflict-handling UX.

**11. Testing Strategy**

- [x] Unit tests for validation, normalization, and dedupe logic.
- [x] Contract tests for all new Tauri commands and schema parsing.
- [x] Store selector tests for filtering/selection behavior.
- [x] UI integration tests for left-search/right-add workflows and bulk actions.
- [x] E2E smoke tests for create/search/edit/delete flow and tab-switch
   responsiveness.
- [x] Regression tests around app startup, diagnostics, and existing tabs.

**12. Rollout and Risk Control**

- [x] Ship behind a config flag first (`ui.features.contacts = true`) for internal
   testing.
- [x] Capture command failures in diagnostics panel with request IDs.
- [x] Add migration-safe storage init with no impact on existing task files.
- [x] Keep rollback simple by isolating contacts state, commands, and storage
   paths.
- [x] Defer online account integration until local UX and performance are stable.

**13. Immediate Execution Order**

- [x] Define V1 schema and command contracts.
- [x] Implement backend persistence and list/add/update/delete commands.
- [x] Add Contacts tab + left-search/right-add UI.
- [x] Wire Zustand slice and selectors with performance constraints.
- [x] Add tests and performance instrumentation.
- [ ] Run internal dogfood pass, then enable by default.

**14. Import From Gmail and Apple iPhone**

- [x] Support import sources in phases:
   - [x] Phase 1: file-based import only (`.vcf`/vCard), including Google and
      iPhone exports.
   - [ ] Phase 2: direct provider import (Google People API / iCloud contacts API)
      if needed later.
- [x] UI entry points:
   - [x] Add `Import Contacts` button in Contacts tab.
   - [x] Import modal with source presets: `Gmail Export`, `iPhone/iCloud Export`,
      `Generic vCard`.
   - [x] Show preview before commit: total rows, valid rows, skipped rows,
      potential duplicates.
- [x] Parser pipeline:
   - [x] Parse vCard versions 2.1/3.0/4.0.
   - [x] Normalize names, phone numbers, emails, notes, org/title, birthday.
   - [x] Keep raw/source metadata for traceability (`import_batch_id`,
      `source_kind`, `source_file_name`).
- [x] Import modes:
   - [x] `Safe import`: add only non-conflicting contacts.
   - [x] `Upsert import`: update existing contacts when match confidence is high.
   - [x] `Review mode`: user confirms each conflict group.
- [x] Import reporting:
   - [x] Created / Updated / Skipped / Failed counts.
   - [x] Downloadable or copyable error summary for malformed cards.
- [x] iPhone-specific handling:
   - [x] Normalize Apple labels and multi-value fields.
   - [x] Handle image/photo blobs as deferred (store reference first; full avatar
      handling later).
- [x] Gmail-specific handling:
   - [x] Normalize Google field labels and custom labels.
   - [x] Preserve primary email/phone flags where present.

**15. Merge / Dedup Capability**

- [x] Matching strategy (scored, not binary):
   - [x] Strong signals: exact normalized email, exact normalized phone.
   - [x] Medium signals: same name + same org, same name + shared domain.
   - [x] Weak signals: fuzzy name only (never auto-merge on weak signal alone).
- [x] Dedup workflow:
   - [x] `Dedup Center` view lists candidate duplicate groups with confidence
      score.
   - [x] Group details show side-by-side field comparison.
   - [x] User actions: `Merge`, `Keep Separate`, `Ignore`.
- [x] Merge rules:
   - [x] Prefer non-empty values over empty.
   - [x] Preserve all unique phones/emails/websites.
   - [x] Choose primary value by confidence/recency/user override.
   - [x] Keep merge audit trail (`merged_from_ids`, timestamp, operator).
- [x] Safety controls:
   - [x] Preview merged result before commit.
   - [x] Undo merge (single-step and batch undo).
   - [x] Soft-delete redundant records first; hard purge later.
- [x] Auto-dedup at import:
   - [x] Run matcher during import and route conflicts to review queue.
   - [x] Never auto-merge low-confidence groups.

**16. Data Model Additions**

- [x] `ContactImportBatch`: id, source type, file name, imported_at, stats.
- [x] `ContactIdentityFingerprint`: normalized name/email/phone hashes for
   matching.
- [x] `MergeAudit`: target_contact_id, source_contact_ids, merge_payload,
   created_at.
- [x] `DedupDecision`: candidate_group_id, decision (`merged/ignored/separate`),
   actor, timestamp.

**17. Command/API Additions**

- [x] `contacts_import_preview(args)` -> parse + normalize + candidate conflicts.
- [x] `contacts_import_commit(args)` -> apply creates/updates with summary.
- [x] `contacts_dedupe_candidates(args)` -> fetch grouped duplicate candidates.
- [x] `contacts_merge(args)` -> merge selected contacts with chosen field
   resolutions.
- [x] `contacts_merge_undo(args)` -> undo a merge transaction.

**18. Milestone Updates**

- [x] Add new milestone between current M2 and M3:
   - [x] **M2.5 Import + Dedup Foundation**
   - [x] vCard parser, import preview/commit, candidate scoring, manual merge UI.
- [x] M3 then includes polish + advanced merge heuristics + undo hardening.

**19. Testing Additions**

- [x] Fixture set: real-world Gmail and iPhone export samples (anonymized).
- [x] Parser tests for vCard variants and malformed cards.
- [x] Matching accuracy tests with precision/recall thresholds.
- [x] Merge correctness tests (field union, primary selection, audit trail, undo).
- [x] E2E: import -> review conflicts -> merge -> verify search/list results.
