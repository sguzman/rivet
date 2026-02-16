# Rivet GUI

Desktop GUI for Rivet core.

## Stack

- Rust backend with Tauri (`src-tauri`)
- Rust frontend with Yew (`ui`)
- Plain CSS theme and layout
- Shared DTO contract in `rivet_gui_shared`

## Runtime Data Path

The GUI backend writes to:

- `RIVET_GUI_DATA` env var if set
- otherwise `./.rivet_gui_data`

## Commands Exposed from Backend

- `tasks_list`
- `task_add`
- `task_update`
- `task_done`
- `task_delete`

All commands map to `rivet_core` task persistence logic.

## Dev

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk tauri-cli
cd crates/rivet-gui/src-tauri
cargo tauri dev
```
