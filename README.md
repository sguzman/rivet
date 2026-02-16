# Rivet

Rivet is a Rust-first Taskwarrior port with two layers:

- A CLI-compatible core (`task`) focused on Taskwarrior-style workflows.
- A desktop GUI layer built with Rust + Yew + CSS + Tauri on top of the same core data model.

See `ROADMAP.md` for the full milestone plan toward comprehensive parity.

## Workspace Layout

- `crates/rivet-core`: task engine, parsing, datastore, filters, renderer, command dispatch.
- `crates/rivet-cli`: `task` binary.
- `crates/rivet-parity`: parity harness that compares Rivet results to Taskwarrior.
- `crates/rivet-gui-shared`: shared DTOs for GUI frontend/backend.
- `crates/rivet-gui/src-tauri`: Tauri backend wired to `rivet-core`.
- `crates/rivet-gui/ui`: Yew frontend + CSS.

## Implemented CLI Commands

- `add`
- `append`
- `prepend`
- `list`
- `next`
- `info`
- `modify`
- `done`
- `delete`
- `export`
- `import`
- `projects`
- `tags`
- `_commands`
- `_show`
- `_unique`

## Core Behavior

- Taskwarrior-style argument parsing (`task <filter> <command> <args>`).
- `taskrc` loading with `include` support.
- Runtime `rc.*` overrides (`--rc` and positional `rc.foo=bar`).
- `TASKRC=/dev/null` behavior.
- Data storage in JSONL files:
  - `pending.data`
  - `completed.data`
- Field support for:
  - `project`, `tags`, `priority`, `due`, `scheduled`, `wait`, `depends`.
- Date expression support:
  - `now`, `today`, `tomorrow`, `yesterday`, `+Nd`, `+Nh`, `+Nm`, RFC3339, `YYYY-MM-DD`, `YYYY-MM-DDTHH:MM`, Taskwarrior export format.
- Colorized tabular rendering in terminal output.

## Logging (Tracing)

Tracing is wired across CLI parsing, config resolution, datastore operations, filtering, command dispatch, parity execution, and GUI backend.

Examples:

```bash
RUST_LOG=rivet_core=debug task add "debug me" project:rivet +trace due:tomorrow
RUST_LOG=trace task +urgent list
```

## Build and Test

### Core / CLI / Parity

```bash
cargo build
cargo test
```

### GUI Shared + Frontend + Backend checks

```bash
cargo check -p rivet_gui_shared
cargo check -p rivet_gui_ui --target wasm32-unknown-unknown
cargo check -p rivet_gui_tauri
```

## Parity Harness

A scenario-based parity harness is included at `crates/rivet-parity`.

Default scenario:

- `crates/rivet-parity/scenarios/basic_flow.json`
- `crates/rivet-parity/scenarios/lifecycle_delete.json`
- `crates/rivet-parity/scenarios/waiting_and_modify.json`
- `crates/rivet-parity/scenarios/append_prepend.json`

Run candidate-only:

```bash
cargo run -p rivet_parity -- --skip-reference
```

Run against Taskwarrior if installed:

```bash
cargo run -p rivet_parity -- \
  --candidate-bin target/debug/task \
  --reference-bin /usr/bin/task
```

Run all included scenarios explicitly:

```bash
cargo run -p rivet_parity -- \
  --candidate-bin target/debug/task \
  --reference-bin task \
  --scenario crates/rivet-parity/scenarios/basic_flow.json \
  --scenario crates/rivet-parity/scenarios/lifecycle_delete.json \
  --scenario crates/rivet-parity/scenarios/waiting_and_modify.json \
  --scenario crates/rivet-parity/scenarios/append_prepend.json
```

The harness reports per-scenario bucket parity (`pending`, `completed`, `deleted`) and an overall score using Jaccard similarity over canonicalized exported tasks.

## GUI Development

Prerequisites:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk tauri-cli
```

Run desktop app:

```bash
cd crates/rivet-gui/src-tauri
cargo tauri dev
```

Frontend styling lives at:

- `crates/rivet-gui/ui/assets/app.css`

## Notes

This is a comprehensive port foundation with extensive instrumentation and a parity measurement workflow. Full Taskwarrior parity (all reports, all grammar/operators, recurrence engine, sync/hooks, undo model) can be layered onto the existing architecture incrementally.
