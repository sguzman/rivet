# Tauri Command Contracts

This document is the canonical command-level contract between the React UI shell (`crates/rivet-gui/ui/web`) and the Tauri command bridge (`crates/rivet-gui/src-tauri/src/commands`).

All commands support an optional `request_id` argument for correlation logging.

## Envelope and Correlation

- Invoke style:
  - Frontend sends `invoke(command, { args: payload, request_id })` for payload commands.
  - Frontend sends `invoke(command, { request_id })` for no-arg commands.
- Correlation:
  - Frontend generates a UUID `request_id` per invoke.
  - Frontend logs command start/success/error with `request_id`.
  - Backend logs received `request_id` in command handlers.

## Commands

### `tasks_list`

- Request `args`:
  - `query: string | null`
  - `status: "Pending" | "Completed" | "Deleted" | "Waiting" | null`
  - `project: string | null`
  - `tag: string | null`
- Response:
  - `TaskDto[]`
- Errors:
  - string message from backend datastore/query failure.

### `task_add`

- Request `args`:
  - `title: string`
  - `description: string`
  - `project: string | null`
  - `tags: string[]`
  - `priority: "Low" | "Medium" | "High" | null`
  - `due: string | null`
  - `wait: string | null`
  - `scheduled: string | null`
- Response:
  - `TaskDto`
- Errors:
  - string message from backend validation/store failure.

### `task_update`

- Request `args`:
  - `uuid: string`
  - `patch:`
    - `title?: string`
    - `description?: string`
    - `project?: string | null`
    - `tags?: string[]`
    - `priority?: "Low" | "Medium" | "High" | null`
    - `due?: string | null`
    - `wait?: string | null`
    - `scheduled?: string | null`
- Response:
  - `TaskDto`
- Errors:
  - string message from backend update failure.

### `task_done`

- Request `args`:
  - `uuid: string`
- Response:
  - `TaskDto`
- Errors:
  - string message from backend update failure.

### `task_delete`

- Request `args`:
  - `uuid: string`
- Response:
  - `void`
- Errors:
  - string message from backend delete failure.

### `external_calendar_sync`

- Request `args`:
  - `id: string`
  - `name: string`
  - `color: string`
  - `location: string`
  - `refresh_minutes: number`
  - `enabled: boolean`
  - `imported_ics_file: boolean`
  - `read_only: boolean`
  - `show_reminders: boolean`
  - `offline_support: boolean`
- Response:
  - `calendar_id: string`
  - `created: number`
  - `updated: number`
  - `deleted: number`
  - `remote_events: number`
  - `refresh_minutes: number`
- Errors:
  - string message for fetch/parse/sync failure.
  - imported ICS sources intentionally return an error asking for re-import.

### `external_calendar_import_ics`

- Request `args`:
  - `source:` same shape as `external_calendar_sync` source.
  - `ics_text: string`
- Response:
  - same shape as `external_calendar_sync`.
- Errors:
  - string message for empty file/parse/sync failure.

### `config_snapshot`

- Request:
  - no `args`
- Response:
  - JSON object parsed from `rivet.toml`.
- Errors:
  - string message when file is missing or unreadable.

### `tag_schema_snapshot`

- Request:
  - no `args`
- Response:
  - JSON object parsed from `crates/rivet-gui/ui/assets/tags.toml`.
- Errors:
  - string message when file is missing or unreadable.

### `ui_log`

- Request `args`:
  - `event: string`
  - `detail: string`
- Response:
  - `void`
- Errors:
  - generally none; errors are still surfaced if logging bridge fails.

## Frontend Runtime Validation

The React API client validates command I/O with `zod` schemas in:

- `crates/rivet-gui/ui/web/api/schemas.ts`

Validation is enforced before values are accepted into UI state, and validation failures are reported as invoke errors.
