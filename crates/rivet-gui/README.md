# Rivet GUI

Desktop GUI for Rivet core.

## Stack

- Rust backend with Tauri (`src-tauri`)
- TypeScript/React frontend shell (`ui/web`)
- Tailwind + MUI for layout and component primitives
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
cargo install tauri-cli
pnpm install
cd crates/rivet-gui/src-tauri
cargo tauri dev
```
