# GUI Parity Checklist

Use this checklist for migration parity signoff between legacy Yew behavior and React shell behavior.

Automated coverage references:

- Command/schema contracts: `web/api/schemas.test.ts`
- Tag movement semantics: `web/lib/tags.test.ts`
- Selector + large dataset checks: `web/store/selectors.test.ts`
- Modal/save regression behaviors: `web/store/useAppStore.test.ts`

## Tasks Workspace

- [ ] Load tasks without dead clicks.
- [ ] Search filter updates result list correctly.
- [ ] Completion filter (`all/pending/waiting/completed/deleted`) works.
- [ ] Project filter facets and filtering work.
- [ ] Tag filter facets and filtering work.
- [ ] Priority filter works.
- [ ] Due filter (`all/has/no due`) works.
- [ ] Add task modal opens from header button and keyboard shortcut.
- [ ] Add task validation blocks empty title.
- [ ] Save task completes and closes modal.
- [ ] Done action updates status.
- [ ] Delete action removes task.
- [ ] Theme toggle persists after reload.

## Kanban Workspace

- [ ] Board create/select/rename/delete works.
- [ ] Add task to board locks board selection.
- [ ] New board task receives lane tag default.
- [ ] Card move control updates lane tag.
- [ ] Drag/drop lane move works.
- [ ] Card density toggle works.
- [ ] Kanban filter panel applies status/project/tag/priority/due.
- [ ] Board deletion cleans board tags from affected tasks.

## Calendar Workspace

- [ ] Year/quarter/month/week/day views render.
- [ ] Prev/Today/Next navigation works.
- [ ] Year and quarter month selection navigates to month view.
- [ ] Month week shortcut navigates to week view.
- [ ] Marker shapes/colors match source policy.
- [ ] Stats panel matches current period totals.
- [ ] Current-period task list respects calendar filter.

## External Calendars

- [ ] Add source modal validates required fields.
- [ ] Edit source persists changed values.
- [ ] Delete source modal removes selected source.
- [ ] Import ICS file creates/updates source tasks.
- [ ] Sync enabled calendars updates created/updated/deleted counts.
- [ ] Imported ICS source has sync disabled and refresh locked.

## Settings and Diagnostics

- [ ] Settings dialog opens/closes.
- [ ] Notification toggles update persisted config.
- [ ] Notification permission request path works.
- [ ] Diagnostics panel shows recent command failures in dev mode.

## Regression Scenarios

- [ ] No dead-click interactions.
- [ ] No modal lockups.
- [ ] No save hangs.
- [ ] No drag instability.
- [ ] No cascading UI freeze after failed command.

## Test Matrix

- [ ] Linux + Wayland
- [ ] Day theme
- [ ] Night theme
- [ ] Small dataset (<50 tasks)
- [ ] Large dataset (>500 tasks)
