# Wayland + Tauri Troubleshooting

## Symptoms and Checks

## Buttons Hover But Do Not Click

Checks:

1. Confirm no fullscreen overlay or modal backdrop intercepting pointer events.
2. Verify command loop is not blocked by hung invoke.
3. Inspect dev diagnostics panel for repeated invoke errors/timeouts.

Actions:

1. Run with fresh state:
   - clear browser local storage keys used by UI
2. Check `logs/` (dev mode) for command start/error correlation by `request_id`.

## UI Feels Laggy

Checks:

1. Verify task list virtualization is active.
2. Check expensive filter recomputation frequency via `render.profile` logs.
3. Inspect large dataset scenarios for repeated state updates.

Actions:

1. Confirm debounce in search/filter flows.
2. Batch multi-task updates instead of per-task state writes.

## Tauri Command Hangs (e.g. Saving...)

Checks:

1. Frontend invoke timeout logs in diagnostics.
2. Backend command logs for same `request_id`.

Actions:

1. Ensure command argument names match handler signature.
2. Validate payload shape with zod schema before invoke.
3. Verify capability permissions for the invoked command.

## Dev Log File Not Created

Checks:

1. `rivet.toml` mode is `dev`.
2. logging directory exists or is creatable by process.

Actions:

1. Ensure:
   - `app.mode = "dev"` or top-level `mode = "dev"`
   - `logging.directory` points to writable path
2. If file sink fails, backend falls back to stderr; inspect terminal output.

## Icons Not Showing In Desktop Shell

Checks:

1. Tauri bundle icons configured in `tauri.conf.json`.
2. Desktop entry cache may need refresh on Linux.

Actions:

1. Rebuild app bundle and reinstall desktop entry.
2. Validate generated `.desktop` points to existing icon resource.
