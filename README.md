# Rivet

Rivet is a Rust-first Taskwarrior port with both a CLI-compatible core and a desktop GUI built on the same task model.

## Intent

Recreate serious Taskwarrior workflows in Rust while preserving CLI muscle memory and opening the door to a richer desktop experience on top of the same engine.

## Ambition

The parity matrix, GUI workspace members, and roadmap make the long-term ambition explicit: build a robust Rust-native Taskwarrior successor with strong compatibility guarantees and a modern GUI layer.

## Current Status

The workspace already implements a large command surface, parity tooling, shared GUI types, frontend/backend integration, and extensive docs. This is one of the more mature repos in the workspace collection.

## Core Capabilities Or Focus Areas

- Taskwarrior-style CLI core in Rust.
- Dedicated parity harness for behavioral comparison.
- Shared types between core and GUI layers.
- Tauri backend plus React frontend for the desktop UI.
- Rich docs around parity, roadmap, and branding.

## Project Layout

- `crates/rivet-core/`: Taskwarrior-style engine, parser, datastore, filters, reports, and command dispatch.
- `crates/rivet-cli/`: CLI binary surface for the `task`-style workflow.
- `crates/rivet-parity/`: compatibility harness for comparing Rivet against Taskwarrior.
- `crates/rivet-gui-shared/`: shared DTOs and contracts between frontend and backend.
- `crates/rivet-gui/src-tauri/`: desktop backend for the Tauri-based GUI layer.
- `branding/`: branding, mascot, and visual identity assets.
- `crates/`: workspace member crates grouped by subsystem.
- `docs/`: project documentation, reference material, and roadmap notes.
- `tests/`: automated tests, fixtures, or parity scenarios.
- `Cargo.toml`: crate or workspace manifest and the first place to check for package structure.

## Setup And Requirements

- Rust toolchain.
- Node.js and `pnpm` for GUI work.
- Tauri prerequisites if running the desktop application.

## Build / Run / Test Commands

```bash
cargo build --workspace
cargo test --workspace
pnpm install
pnpm ui:build
cargo tauri dev --manifest-path crates/rivet-gui/src-tauri/Cargo.toml
```

## Notes, Limitations, Or Known Gaps

- CLI parity is a core contract and should be treated carefully when evolving internals.
- The GUI is an additional product surface, not a replacement for the command-line compatibility work.

## Next Steps Or Roadmap Hints

- Continue using the parity matrix and harness as the guardrail for CLI expansion.
- Keep GUI-specific behavior cleanly layered on top of the shared task model.
