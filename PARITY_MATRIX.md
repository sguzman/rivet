# Rivet Parity Matrix (Taskwarrior 3.4.2)

Legend:

- `implemented`: behavior exists and has parity coverage in harness/tests
- `partial`: behavior exists but is incomplete or not yet fully parity-verified
- `missing`: not implemented yet

Machine-readable source of this matrix: `tests/parity_map.json`

## Command Surface

| Area | Command / Capability | Status | Notes |
|---|---|---|---|
| Task lifecycle | `add` | implemented | Core modifiers supported for project/tags/priority/due/scheduled/wait/depends |
| Task lifecycle | `list` / `next` | implemented | Filtered listing with colorized output |
| Task lifecycle | `info` | implemented | Supports id/uuid/filter selection |
| Task lifecycle | `modify` | implemented | Modifier application across filtered set |
| Task lifecycle | `append` / `prepend` | implemented | Included in parity scenarios |
| Task lifecycle | `done` | implemented | Moves tasks to `completed.data` |
| Task lifecycle | `delete` (soft) | implemented | Marks deleted in pending |
| Data I/O | `export` | implemented | JSON export for filtered tasks |
| Data I/O | `import` | partial | JSON array/JSONL supported; broader edge-case compatibility pending |
| Discovery | `projects` / `tags` | implemented | Basic unique listing |
| Discovery | `_commands` / `_show` / `_unique` | implemented | Minimal completion-oriented behavior |
| Convenience | numeric shortcut (`task 1`) | implemented | Mapped to `info` selection |
| Task state | `start` / `stop` | missing | Planned in roadmap M3 |
| Task state | `undo` | missing | Planned in roadmap M3 / M7 |
| Task metadata | `annotate` / `denotate` | missing | Planned in roadmap M3 |
| Task metadata | `duplicate` | missing | Planned in roadmap M3 |
| Logging/history | `log` command parity | missing | Planned in roadmap M3 |
| Contexts | context command set | missing | Planned in roadmap M3 |
| Reports | custom report command surface | partial | Basic list rendering exists; full report engine pending |
| Sync | taskserver sync | missing | Optional feature-gated scope (M8) |

## Filter Grammar

| Capability | Status | Notes |
|---|---|---|
| Tag include/exclude (`+tag`, `-tag`) | implemented | Covered in parity scenarios |
| ID/UUID selection | implemented | Numeric and UUID terms supported |
| `project:` equality | implemented | Basic equality matching |
| `status:pending/completed/deleted/waiting` | implemented | Waiting semantics aligned to Taskwarrior model |
| `due.before:` / `due.after:` | implemented | Date parser-backed comparisons |
| Free-text description contains | implemented | Case-insensitive contains |
| Boolean operators (`and/or/not`) | missing | M4 |
| Parentheses precedence | missing | M4 |
| Rich attribute comparisons/ranges | partial | Limited date comparisons only |
| Virtual tags full compatibility | missing | M4 |

## Date/Time Semantics

| Capability | Status | Notes |
|---|---|---|
| `now/today/tomorrow/yesterday` parsing | implemented | Core parser support |
| Relative offsets (`+Nd/+Nh/+Nm`) | implemented | Core parser support |
| RFC3339 + Taskwarrior timestamp format | implemented | Roundtrip serializer/deserializer in core |
| Timezone/DST edge parity | partial | Needs dedicated parity scenarios |

## Config / Runtime Behavior

| Capability | Status | Notes |
|---|---|---|
| `taskrc` loading | implemented | Supports include recursion |
| `TASKRC=/dev/null` behavior | implemented | Config suppression supported |
| `rc.*` overrides | implemented | Positional + `--rc` supported |
| Full taskrc/report config compatibility | partial | Core keys supported; full report/config spectrum pending |

## Persistence Model

| Capability | Status | Notes |
|---|---|---|
| JSONL datastore (`pending.data` / `completed.data`) | implemented | Atomic writes in core datastore |
| Task status transitions | implemented | Pending/completed/deleted/waiting semantics aligned for covered paths |
| Undo journal persistence | missing | M7 |
| Hook side-effects persistence model | missing | M7 |

## Reporting / Rendering

| Capability | Status | Notes |
|---|---|---|
| Table rendering with ANSI color | implemented | Overdue highlighting and terminal-aware color |
| Full `report.*` columns/labels/sorting parity | missing | M5 |
| Urgency scoring parity | missing | M5 |
| Full formatting parity with Taskwarrior reports | partial | Basic display exists |

## GUI Layer (Rust + Yew + CSS + Tauri)

| Capability | Status | Notes |
|---|---|---|
| Tauri backend commands (`tasks_list`, `task_add`, `task_update`, `task_done`, `task_delete`) | implemented | Wired to core datastore logic |
| Yew task list/details/create/edit shell | implemented | Functional desktop UI skeleton |
| Route-level views (`inbox/projects/tags/completed/settings`) | partial | Sidebar exists; full routed pages pending |
| Bulk actions / advanced report views | missing | M9 |
| End-to-end GUI integration tests | missing | M9 |

## Parity Harness Coverage

| Suite | Status | Notes |
|---|---|---|
| `basic_flow` | implemented | Candidate vs Taskwarrior parity passing |
| `lifecycle_delete` | implemented | Candidate vs Taskwarrior parity passing |
| `waiting_and_modify` | implemented | Candidate vs Taskwarrior parity passing |
| `append_prepend` | implemented | Candidate vs Taskwarrior parity passing |
| Error-path/invalid-input suites | missing | M2 |
| Config/include/override suites | partial | Minimal coverage, needs expansion |
| Report formatting suites | missing | M2/M5 |

## Next Matrix Actions

1. Add missing command rows to harness-backed scenarios (start/stop/undo/annotate/etc.) as they are implemented.
2. Expand filter grammar rows into parser-level test cases tied to M4.
3. Add report engine rows tied to concrete `report.*` fixtures and expected outputs.
4. Keep each row linked to a scenario/test filename once coverage exists.
