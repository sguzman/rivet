# GUI Parity Checklist

Use this checklist for migration parity signoff between legacy Yew behavior and React shell behavior.

Automated coverage references:

- Command/schema contracts: `web/api/schemas.test.ts`
- Tag movement semantics: `web/lib/tags.test.ts`
- Selector + large dataset checks: `web/store/selectors.test.ts`
- Modal/save regression behaviors: `web/store/useAppStore.test.ts`

## Tasks Workspace

- [x] Load tasks without dead clicks.
- [x] Search filter updates result list correctly.
- [x] Completion filter (`all/pending/waiting/completed/deleted`) works.
- [x] Project filter facets and filtering work.
- [x] Tag filter facets and filtering work.
- [x] Priority filter works.
- [x] Due filter (`all/has/no due`) works.
- [x] Add task modal opens from header button and keyboard shortcut.
- [x] Add task validation blocks empty title.
- [x] Save task completes and closes modal.
- [x] Done action updates status.
- [x] Delete action removes task.
- [x] Theme toggle persists after reload.

## Kanban Workspace

- [x] Board create/select/rename/delete works.
- [x] Add task to board locks board selection.
- [x] New board task receives lane tag default.
- [x] Card move control updates lane tag.
- [x] Drag/drop lane move works.
- [x] Card density toggle works.
- [x] Kanban filter panel applies status/project/tag/priority/due.
- [x] Board deletion cleans board tags from affected tasks.

## Calendar Workspace

- [x] Year/quarter/month/week/day views render.
- [x] Prev/Today/Next navigation works.
- [x] Year and quarter month selection navigates to month view.
- [x] Month week shortcut navigates to week view.
- [x] Marker shapes/colors match source policy.
- [x] Stats panel matches current period totals.
- [x] Current-period task list respects calendar filter.

## External Calendars

- [x] Add source modal validates required fields.
- [x] Edit source persists changed values.
- [x] Delete source modal removes selected source.
- [x] Import ICS file creates/updates source tasks.
- [x] Sync enabled calendars updates created/updated/deleted counts.
- [x] Imported ICS source has sync disabled and refresh locked.

## Settings and Diagnostics

- [x] Settings dialog opens/closes.
- [x] Notification toggles update persisted config.
- [x] Notification permission request path works.
- [x] Diagnostics panel shows recent command failures in dev mode.

## Regression Scenarios

- [x] No dead-click interactions.
- [x] No modal lockups.
- [x] No save hangs.
- [x] No drag instability.
- [x] No cascading UI freeze after failed command.

## Test Matrix

- [x] Linux + Wayland
- [x] Day theme
- [x] Night theme
- [x] Small dataset (<50 tasks)
- [x] Large dataset (>500 tasks)
