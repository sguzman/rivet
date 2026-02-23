# MUI Component Audit

Audit scope: React shell under `crates/rivet-gui/ui/web`.

## Global Shell

- `AppBar`, `Toolbar`, `Tabs`, `Tab`, `Button`, `Typography`
- dialogs hosted through feature/components and rendered with MUI primitives

## Tasks Workspace

- Filters and forms use `TextField`, `MenuItem`, `Button`
- Panels use `Paper`, `Stack`, `Typography`
- Task list uses `List`, `ListItemButton`, `ListItemText`
- Status/tag chips use `Chip`-based composition
- Details actions use MUI button variants

## Kanban Workspace

- Board management modals use `Dialog`, `DialogTitle`, `DialogContent`, `DialogActions`
- Filters use `TextField` selects and `MenuItem`
- Cards and lane containers use `Paper`, `Box`, `Stack`, `Typography`, `Button`

## Calendar Workspace

- View controls, source actions and filters use `Button`, `TextField`, `MenuItem`
- External source forms and delete confirmation use `Dialog` family components
- Settings in calendar source modal use `Checkbox` + `FormControlLabel`
- Stats and task panels use `Paper`, `Stack`, `Typography`

## Settings/Diagnostics

- Settings dialog uses MUI form controls and button patterns
- Diagnostics panel uses MUI surfaces and controls for dev telemetry

## Findings

- All interactive controls are MUI-based.
- Tailwind is used primarily for layout utility classes and spacing.
- No remaining custom non-MUI form controls are required for parity-critical flows.
