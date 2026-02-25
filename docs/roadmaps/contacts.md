**Contacts Feature Roadmap (Rivet GUI, No CLI Surface)**

**1. Product Scope**

- [ ] Add a new `Contacts` tab to the existing app shell, alongside
   Tasks/Kanban/Calendar.
- [ ] Primary layout requirement: left area for search + browsing contacts, right
   area for adding a new contact.
- [ ] Keep first release local-first and offline-first, with architecture ready for
   online/contact-source aggregation later.
- [ ] Exclude all command-line behavior from scope.

**2. Feature Set Adapted From GNOME Contacts**

- [ ] Include now: contact list, search, create, edit, delete, multiple
   phones/emails, typed fields, optional expanded fields, duplicate-aware
   behavior.
- [ ] Include now: selection mode for multi-delete and future merge/link.
- [ ] Include now: quick actions to launch `mailto:` and `tel:` (where platform
   supports).
- [ ] Include later: source aggregation (local + external), contact
   linking/unlinking, richer profile metadata, avatars, birthday UX.
- [ ] Exclude initially: account setup flows, platform account provisioning,
   advanced sync conflict UIs.

**3. UX Structure**

- [ ] Left pane: search box, filters, result list, optional selection mode toggle,
   bulk actions bar.
- [ ] Right pane: “Add Contact” form by default, with fast create workflow and
   validation.
- [ ] Right pane behavior after selection: switch to “Contact Details/Edit” view
   while preserving an “Add New” button.
- [ ] Empty states: no contacts yet, no search matches, validation errors,
   loading/failure states.
- [ ] Keyboard support: arrow selection, Enter to open, `Cmd/Ctrl+N` add contact,
   `Delete` when in selection mode.

**4. Contact Data Model (V1)**

- [ ] `Contact`: `id`, `display_name`, `given_name`, `family_name`, `nickname`,
   `notes`, `created_at`, `updated_at`.
- [ ] `Phones[]`: `value`, `type` (`mobile/home/work/other`), `is_primary`.
- [ ] `Emails[]`: `value`, `type`, `is_primary`.
- [ ] `Websites[]`, `Birthday` (optional), `Organization` (optional), `Title`
   (optional).
- [ ] `Addresses[]` deferred to V1.1 unless needed immediately.
- [ ] `Source metadata`: `source_id`, `source_kind` (start with `local`),
   `remote_id` nullable.
- [ ] `Merged identity hooks` (for future linking): `link_group_id` nullable.

**5. Backend Architecture (Tauri + Rust)**

- [ ] Add contact domain/state handling in GUI backend first, parallel to existing
   task commands.
- [ ] Persist contacts in dedicated storage, not task records.
- [ ] Recommended storage shape: `contacts.data` JSONL (and optional
   `contacts_deleted.data` if soft delete is needed).
- [ ] Build normalized in-memory indexes for search (name/email/phone tokens) to
   avoid full scans on every keypress.
- [ ] Add DTOs in `rivet-gui-shared` for strict frontend/backend contract typing.
- [ ] Keep design ready for eventual migration into `rivet-core` domain if
   cross-surface reuse is needed later.

**6. Tauri Command Contract (V1)**

- [ ] `contacts_list(args)` with `query`, `limit`, `cursor`, optional `source`,
   optional `updated_after`.
- [ ] `contact_add(args)` with full create payload and server-side validation.
- [ ] `contact_update(args)` patch-style updates.
- [ ] `contact_delete(args)` single delete.
- [ ] `contacts_delete_bulk(args)` bulk delete.
- [ ] `contacts_dedupe_preview(args)` optional early dedupe utility.
- [ ] `contact_open_action(args)` for `mailto:` / `tel:` launch abstraction.
- [ ] Add command correlation + error surfacing exactly like existing command
   telemetry patterns.

**7. Frontend State Design (Zustand)**

- [ ] Create a dedicated `contacts` slice, separate from task/kanban/calendar
   concerns.
- [ ] Store only UI/session state in Zustand, keep canonical data from backend
   responses.
- [ ] State keys: `contacts`, `query`, `selectedContactId`, `selectionIds`,
   `loading`, `error`, `formDraft`, `dirty`.
- [ ] Derived selectors must be memoized and cheap, especially for search results
   and facet counts.
- [ ] Avoid full-list recomputation on every keystroke by using indexed backend
   query + debounced input.
- [ ] Preserve current tab performance discipline: avoid high-frequency bridge
   logging for every input event.

**8. Search and Performance Plan**

- [ ] Search behavior: incremental search over name/email/phone with
   case/diacritic-insensitive matching.
- [ ] Debounce input (150–250ms) and cancel stale in-flight requests.
- [ ] Support pagination or windowing for large sets (start with 200 visible rows
   max + “load more”).
- [ ] Virtualize list rendering if contact counts exceed threshold.
- [ ] Cache recent query results by key for quick tab revisits.
- [ ] Add performance budgets: first result <100ms on warm cache, tab switch <120ms
   target.

**9. Validation and Data Quality**

- [ ] Require at least one of: display name, email, or phone.
- [ ] Email format validation (strict enough to catch obvious errors, not
   RFC-overkill UX).
- [ ] Phone normalization for search but preserve original display formatting.
- [ ] Prevent accidental duplicates with non-blocking warning (“possible duplicate
   exists”).
- [ ] Enforce safe field limits to prevent runaway payloads.

**10. Milestone Plan**

- [ ] **M0 (Foundation)**: DTOs, storage files, command skeleton, tab shell
   integration, empty-state UI.
- [ ] **M1 (Core Usable)**: add/list/search/edit/delete single contact, right-pane
   add form, detail panel.
- [ ] **M2 (Bulk + Quality)**: selection mode, bulk delete, dedupe warnings,
   keyboard shortcuts, polish.
- [ ] **M3 (Parity Enhancements)**: typed fields expansion, communication launch
   actions, richer profile fields.
- [ ] **M4 (Advanced Optional)**: contact linking/unlinking and source-aware
   architecture groundwork.
- [ ] **M5 (Future Expansion)**: online source sync/aggregation and
   conflict-handling UX.

**11. Testing Strategy**

- [ ] Unit tests for validation, normalization, and dedupe logic.
- [ ] Contract tests for all new Tauri commands and schema parsing.
- [ ] Store selector tests for filtering/selection behavior.
- [ ] UI integration tests for left-search/right-add workflows and bulk actions.
- [ ] E2E smoke tests for create/search/edit/delete flow and tab-switch
   responsiveness.
- [ ] Regression tests around app startup, diagnostics, and existing tabs.

**12. Rollout and Risk Control**

- [ ] Ship behind a config flag first (`ui.features.contacts = true`) for internal
   testing.
- [ ] Capture command failures in diagnostics panel with request IDs.
- [ ] Add migration-safe storage init with no impact on existing task files.
- [ ] Keep rollback simple by isolating contacts state, commands, and storage
   paths.
- [ ] Defer online account integration until local UX and performance are stable.

**13. Immediate Execution Order**

- [ ] Define V1 schema and command contracts.
- [ ] Implement backend persistence and list/add/update/delete commands.
- [ ] Add Contacts tab + left-search/right-add UI.
- [ ] Wire Zustand slice and selectors with performance constraints.
- [ ] Add tests and performance instrumentation.
- [ ] Run internal dogfood pass, then enable by default.

**14. Import From Gmail and Apple iPhone**

- [ ] Support import sources in phases:
   - [ ] Phase 1: file-based import only (`.vcf`/vCard), including Google and
      iPhone exports.
   - [ ] Phase 2: direct provider import (Google People API / iCloud contacts API)
      if needed later.
- [ ] UI entry points:
   - [ ] Add `Import Contacts` button in Contacts tab.
   - [ ] Import modal with source presets: `Gmail Export`, `iPhone/iCloud Export`,
      `Generic vCard`.
   - [ ] Show preview before commit: total rows, valid rows, skipped rows,
      potential duplicates.
- [ ] Parser pipeline:
   - [ ] Parse vCard versions 2.1/3.0/4.0.
   - [ ] Normalize names, phone numbers, emails, notes, org/title, birthday.
   - [ ] Keep raw/source metadata for traceability (`import_batch_id`,
      `source_kind`, `source_file_name`).
- [ ] Import modes:
   - [ ] `Safe import`: add only non-conflicting contacts.
   - [ ] `Upsert import`: update existing contacts when match confidence is high.
   - [ ] `Review mode`: user confirms each conflict group.
- [ ] Import reporting:
   - [ ] Created / Updated / Skipped / Failed counts.
   - [ ] Downloadable or copyable error summary for malformed cards.
- [ ] iPhone-specific handling:
   - [ ] Normalize Apple labels and multi-value fields.
   - [ ] Handle image/photo blobs as deferred (store reference first; full avatar
      handling later).
- [ ] Gmail-specific handling:
   - [ ] Normalize Google field labels and custom labels.
   - [ ] Preserve primary email/phone flags where present.

**15. Merge / Dedup Capability**

- [ ] Matching strategy (scored, not binary):
   - [ ] Strong signals: exact normalized email, exact normalized phone.
   - [ ] Medium signals: same name + same org, same name + shared domain.
   - [ ] Weak signals: fuzzy name only (never auto-merge on weak signal alone).
- [ ] Dedup workflow:
   - [ ] `Dedup Center` view lists candidate duplicate groups with confidence
      score.
   - [ ] Group details show side-by-side field comparison.
   - [ ] User actions: `Merge`, `Keep Separate`, `Ignore`.
- [ ] Merge rules:
   - [ ] Prefer non-empty values over empty.
   - [ ] Preserve all unique phones/emails/websites.
   - [ ] Choose primary value by confidence/recency/user override.
   - [ ] Keep merge audit trail (`merged_from_ids`, timestamp, operator).
- [ ] Safety controls:
   - [ ] Preview merged result before commit.
   - [ ] Undo merge (single-step and batch undo).
   - [ ] Soft-delete redundant records first; hard purge later.
- [ ] Auto-dedup at import:
   - [ ] Run matcher during import and route conflicts to review queue.
   - [ ] Never auto-merge low-confidence groups.

**16. Data Model Additions**

- [ ] `ContactImportBatch`: id, source type, file name, imported_at, stats.
- [ ] `ContactIdentityFingerprint`: normalized name/email/phone hashes for
   matching.
- [ ] `MergeAudit`: target_contact_id, source_contact_ids, merge_payload,
   created_at.
- [ ] `DedupDecision`: candidate_group_id, decision (`merged/ignored/separate`),
   actor, timestamp.

**17. Command/API Additions**

- [ ] `contacts_import_preview(args)` -> parse + normalize + candidate conflicts.
- [ ] `contacts_import_commit(args)` -> apply creates/updates with summary.
- [ ] `contacts_dedupe_candidates(args)` -> fetch grouped duplicate candidates.
- [ ] `contacts_merge(args)` -> merge selected contacts with chosen field
   resolutions.
- [ ] `contacts_merge_undo(args)` -> undo a merge transaction.

**18. Milestone Updates**

- [ ] Add new milestone between current M2 and M3:
   - [ ] **M2.5 Import + Dedup Foundation**
   - [ ] vCard parser, import preview/commit, candidate scoring, manual merge UI.
- [ ] M3 then includes polish + advanced merge heuristics + undo hardening.

**19. Testing Additions**

- [ ] Fixture set: real-world Gmail and iPhone export samples (anonymized).
- [ ] Parser tests for vCard variants and malformed cards.
- [ ] Matching accuracy tests with precision/recall thresholds.
- [ ] Merge correctness tests (field union, primary selection, audit trail, undo).
- [ ] E2E: import -> review conflicts -> merge -> verify search/list results.
