# Rivet Full Parity Roadmap

This roadmap tracks work required to move from the current high-parity subset to comprehensive Taskwarrior compatibility.

## Current Snapshot

- Reference target: Taskwarrior `3.4.2`
- Current parity harness status: green on implemented scenario suite
- Scope status: partial command and grammar coverage, not full feature-complete parity
- Working matrix: `PARITY_MATRIX.md`
- Machine-readable matrix data: `tests/parity_map.json`

## Milestones

## M1 - Parity Contract and Inventory

Status: `pending`

- [ ] Build a full feature matrix of Taskwarrior commands/options/modifiers.
- [ ] Classify each item as `implemented`, `partial`, or `missing`.
- [ ] Add owner + priority for each missing or partial area.
- [ ] Define severity labels for mismatches (`critical`, `major`, `minor`).

Exit criteria:

- A complete matrix exists in-repo and is linked from `README.md`.
- Every parity failure can be traced to a matrix item.

## M2 - Parity Harness Expansion

Status: `in_progress`

- [x] Add baseline lifecycle scenarios.
- [x] Add waiting/status interaction scenarios.
- [x] Add append/prepend/modify scenarios.
- [ ] Add negative/error-path scenarios (invalid syntax, invalid IDs, ambiguous abbreviations).
- [ ] Add date/time boundary scenarios (timezone, DST, midnight edge cases).
- [ ] Add rc/config include override scenarios.
- [ ] Add report formatting comparison scenarios.

Exit criteria:

- Harness covers all implemented features and expected error paths.
- Per-feature parity scores and diffs are emitted deterministically.

## M3 - Core Command Surface Completion

Status: `pending`

- [ ] Implement `start` / `stop`.
- [ ] Implement `undo` transactional behavior.
- [ ] Implement `annotate` / `denotate`.
- [ ] Implement `duplicate`.
- [ ] Implement `log` parity behavior.
- [ ] Implement context commands and active context behavior.
- [ ] Align command abbreviation resolution edge cases with Taskwarrior.

Exit criteria:

- Command matrix for local-only commands is fully green.
- Harness scenarios for each command pass against Taskwarrior.

## M4 - Full Filter Grammar

Status: `pending`

- [ ] Implement parser for full boolean grammar with precedence and parentheses.
- [ ] Support complete attribute comparisons and range expressions.
- [ ] Support negation and compound expressions exactly as Taskwarrior resolves them.
- [ ] Support virtual tags and special filter semantics.
- [ ] Add parser/property tests for grammar correctness.

Exit criteria:

- Filter behavior is functionally indistinguishable on test corpus.
- Parser test suite includes fuzz/property coverage.

## M5 - Report Engine Parity

Status: `pending`

- [ ] Implement `report.*` config keys (columns, labels, sorting, limits).
- [ ] Implement urgency/sort parity and column formatting behavior.
- [ ] Implement color rule compatibility and output rules.
- [ ] Match default report output behaviors.

Exit criteria:

- Equivalent configs produce equivalent report selection and ordering.
- Formatting deltas are documented and minimized.

## M6 - Advanced Task Semantics

Status: `pending`

- [ ] Implement recurrence (`recur`, `until`, masks, instance generation).
- [ ] Implement complete dependency resolution semantics.
- [ ] Validate lifecycle transitions across wait/scheduled/start/stop/done flows.
- [ ] Add regression scenarios for recurring and dependency interactions.

Exit criteria:

- Recurrence and dependency scenario suites match Taskwarrior behavior.

## M7 - Undo, Hooks, and Extensibility

Status: `pending`

- [ ] Add durable undo journal with proper transaction boundaries.
- [ ] Implement hook invocation points and safe execution model.
- [ ] Implement failure/rollback behavior for hook errors.
- [ ] Add compatibility tests for hook-triggered modifications.

Exit criteria:

- Undo behavior matches expected Taskwarrior semantics.
- Hook lifecycle behavior is documented and tested.

## M8 - Sync / Taskserver (Optional Feature Gate)

Status: `pending`

- [ ] Design feature-gated sync module.
- [ ] Implement protocol compatibility layer.
- [ ] Implement conflict resolution and recovery behavior.
- [ ] Add integration tests with controlled sync fixtures.

Exit criteria:

- Sync can be enabled safely and passes integration suites.
- Feature remains optional and isolated from non-sync users.

## M9 - GUI Completion on Stable Core

Status: `pending`

- [ ] Add route-level views (`inbox`, `projects`, `tags`, `completed`, `settings`).
- [ ] Add robust create/edit flows including date parsing feedback.
- [ ] Add bulk actions and multi-select workflows.
- [ ] Add report-backed list/table views driven by core APIs.
- [ ] Add end-to-end GUI smoke tests.

Exit criteria:

- GUI behavior maps 1:1 to core task operations.
- GUI remains a thin client over core logic.

## M10 - Stabilization and Release Readiness

Status: `pending`

- [ ] Add cross-platform CI matrix (Linux/macOS/Windows).
- [ ] Add performance benchmarks and profiling baselines.
- [ ] Add migration and compatibility checks for existing task data.
- [ ] Finalize packaging/release automation.
- [ ] Publish operator documentation and known deviations.

Exit criteria:

- CI is green across supported targets.
- Release process is reproducible and documented.

## Tracking Metrics

- Global parity score target: `>= 0.99` on full scenario suite.
- Critical mismatch count target: `0`.
- Regression policy: no new mismatches in previously green scenarios.

## Recommended Execution Order

1. M1 parity contract
2. M2 harness expansion
3. M3 command completion
4. M4 filter grammar
5. M5 report engine
6. M6 advanced semantics
7. M7 undo/hooks
8. M9 GUI completion
9. M10 stabilization
10. M8 sync (feature-gated, optional)
