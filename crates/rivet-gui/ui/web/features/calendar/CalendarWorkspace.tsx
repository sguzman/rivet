import { useEffect, useMemo, useRef, useState } from "react";
import type { ChangeEvent } from "react";

import AddIcon from "@mui/icons-material/Add";
import DeleteIcon from "@mui/icons-material/Delete";
import EditIcon from "@mui/icons-material/Edit";
import SyncIcon from "@mui/icons-material/Sync";
import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Checkbox from "@mui/material/Checkbox";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import FormControlLabel from "@mui/material/FormControlLabel";
import IconButton from "@mui/material/IconButton";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

import { TagChip } from "../../components/TagChip";
import {
  addDays,
  calendarDateFromIso,
  calendarDateToIso,
  calendarMonthGridStart,
  calendarTitleForView,
  collectCalendarDueTasks,
  entriesForDate,
  firstDayOfMonth,
  formatDueDateTime,
  markersForDate,
  monthWeekStarts,
  periodStats,
  periodTasks,
  quarterMonths,
  resolveCalendarConfig,
  startOfWeek,
  toCalendarDate,
  weekdayLabels
} from "../../lib/calendar";
import { CAL_SOURCE_TAG_KEY, firstTagValue } from "../../lib/tags";
import { useBoardColorMap, useExternalCalendarColorMap } from "../../store/useAppStore";
import { useCalendarWorkspaceSlice } from "../../store/slices";
import type { ExternalCalendarCacheEntry, ExternalCalendarSource } from "../../types/core";
import type { CalendarTaskMarker, CalendarViewMode } from "../../types/ui";

function MarkerDots(props: { markers: CalendarTaskMarker[]; limit: number }) {
  if (props.markers.length === 0) {
    return null;
  }
  const capped = props.markers.slice(0, props.limit);
  const overflow = props.markers.length - capped.length;
  return (
    <div className="calendar-markers">
      {capped.map((marker, index) => (
        <span
          key={`${marker.shape}-${marker.color}-${index}`}
          className={`calendar-marker ${marker.shape}`}
          style={{ ["--marker-color" as string]: marker.color }}
        />
      ))}
      {overflow > 0 ? <span className="calendar-overflow">+{overflow}</span> : null}
    </div>
  );
}

function ExternalCalendarCard(props: {
  source: ExternalCalendarSource;
  busy: boolean;
  onSync: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  return (
    <Paper variant="outlined" className="p-2">
      <Stack spacing={0.9}>
        <Stack direction="row" spacing={1} alignItems="center">
          <span className="calendar-color-dot" style={{ backgroundColor: props.source.color }} />
          <Typography variant="subtitle2" className="min-w-0 flex-1 truncate">
            {props.source.name}
          </Typography>
          <Typography variant="caption" className="rounded-md border border-current/15 px-1.5 py-0.5">
            {props.source.enabled ? "enabled" : "disabled"}
          </Typography>
        </Stack>
        <Typography variant="caption" color="text.secondary" className="truncate">
          {props.source.location}
        </Typography>
        <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap>
          <Typography variant="caption" className="rounded-md border border-current/15 px-1.5 py-0.5">
            {props.source.refresh_minutes === 0 ? "refresh:off" : `refresh:${props.source.refresh_minutes}m`}
          </Typography>
          {props.source.imported_ics_file ? (
            <Typography variant="caption" className="rounded-md border border-current/15 px-1.5 py-0.5">
              imported:file
            </Typography>
          ) : null}
        </Stack>
        <Stack direction="row" spacing={0.75}>
          <Button
            size="small"
            variant="outlined"
            disabled={props.busy || props.source.imported_ics_file}
            onClick={props.onSync}
            startIcon={<SyncIcon fontSize="small" />}
          >
            Sync
          </Button>
          <IconButton size="small" aria-label={`Edit ${props.source.name}`} onClick={props.onEdit}>
            <EditIcon fontSize="small" />
          </IconButton>
          <IconButton size="small" color="error" aria-label={`Delete ${props.source.name}`} onClick={props.onDelete}>
            <DeleteIcon fontSize="small" />
          </IconButton>
        </Stack>
      </Stack>
    </Paper>
  );
}

export function CalendarWorkspace() {
  const {
    tasks,
    runtimeConfig,
    calendarView,
    calendarFocusDateIso,
    calendarTaskFilter,
    externalCalendars,
    externalBusy,
    externalLastSync,
    error,
    setCalendarView,
    shiftCalendarFocus,
    setCalendarTaskFilter,
    setCalendarConfigToggle,
    navigateCalendar,
    openNewExternalCalendar,
    saveExternalCalendarSource,
    deleteExternalCalendarSource,
    syncExternalCalendarSource,
    syncAllExternalCalendars,
    importExternalCalendarFile,
    listExternalCalendarCachedEntries,
    importExternalCalendarFromCache
  } = useCalendarWorkspaceSlice();

  const boardColorMap = useBoardColorMap();
  const calendarColorMap = useExternalCalendarColorMap();
  const config = useMemo(() => resolveCalendarConfig(runtimeConfig), [runtimeConfig]);
  const focus = useMemo(() => calendarDateFromIso(calendarFocusDateIso), [calendarFocusDateIso]);
  const title = useMemo(
    () => calendarTitleForView(calendarView, focus, config.policies.week_start),
    [calendarView, focus, config.policies.week_start]
  );

  const allDueEntries = useMemo(() => {
    return collectCalendarDueTasks(tasks, config, boardColorMap, calendarColorMap);
  }, [tasks, config, boardColorMap, calendarColorMap]);

  const currentPeriodEntries = useMemo(() => {
    return periodTasks(allDueEntries, calendarView, focus, config.policies.week_start);
  }, [allDueEntries, calendarView, focus, config.policies.week_start]);

  const stats = useMemo(() => periodStats(currentPeriodEntries), [currentPeriodEntries]);

  const calendarNameMap = useMemo(() => {
    const map = new Map<string, string>();
    for (const source of externalCalendars) {
      map.set(source.id, source.name);
    }
    return map;
  }, [externalCalendars]);

  const calendarFilterOptions = useMemo(() => {
    const ids = new Set<string>();
    for (const entry of currentPeriodEntries) {
      const calendarId = firstTagValue(entry.task.tags, CAL_SOURCE_TAG_KEY);
      if (calendarId) {
        ids.add(calendarId);
      }
    }
    return [...ids].sort().map((id) => ({
      id,
      label: calendarNameMap.get(id) ?? `Calendar ${id}`
    }));
  }, [calendarNameMap, currentPeriodEntries]);

  const [deEmphasizePastPeriods, setDeEmphasizePastPeriods] = useState(config.toggles.de_emphasize_past_periods);
  const [filterTasksBeforeNow, setFilterTasksBeforeNow] = useState(config.toggles.filter_tasks_before_now);
  const [hidePastMarkers, setHidePastMarkers] = useState(config.toggles.hide_past_markers);
  const [nowUtcMs, setNowUtcMs] = useState(() => Date.now());
  const nowLocal = useMemo(() => nowDateTime(config.timezone, nowUtcMs), [config.timezone, nowUtcMs]);
  const todayLocal = useMemo(
    () => toCalendarDate(nowLocal.year, nowLocal.month, nowLocal.day),
    [nowLocal.day, nowLocal.month, nowLocal.year]
  );
  const today = useMemo(
    () => ({ year: nowLocal.year, month: nowLocal.month, day: nowLocal.day }),
    [nowLocal.day, nowLocal.month, nowLocal.year]
  );
  const todayMonthStart = useMemo(
    () => firstDayOfMonth(nowLocal.year, nowLocal.month),
    [nowLocal.month, nowLocal.year]
  );

  useEffect(() => {
    setDeEmphasizePastPeriods(config.toggles.de_emphasize_past_periods);
    setFilterTasksBeforeNow(config.toggles.filter_tasks_before_now);
    setHidePastMarkers(config.toggles.hide_past_markers);
  }, [
    config.toggles.de_emphasize_past_periods,
    config.toggles.filter_tasks_before_now,
    config.toggles.hide_past_markers
  ]);

  const markerEntries = useMemo(() => {
    if (!hidePastMarkers) {
      return allDueEntries;
    }
    return allDueEntries.filter((entry) => entry.dueUtcMs >= nowUtcMs);
  }, [allDueEntries, hidePastMarkers, nowUtcMs]);

  const visiblePeriodEntries = useMemo(() => {
    const byCalendar = currentPeriodEntries.filter((entry) => {
      if (calendarTaskFilter === "__all__") {
        return true;
      }
      if (calendarTaskFilter === "__none__") {
        return firstTagValue(entry.task.tags, CAL_SOURCE_TAG_KEY) === null;
      }
      return firstTagValue(entry.task.tags, CAL_SOURCE_TAG_KEY) === calendarTaskFilter;
    });
    if (filterTasksBeforeNow) {
      return byCalendar.filter((entry) => entry.dueUtcMs >= nowUtcMs);
    }
    return byCalendar;
  }, [calendarTaskFilter, currentPeriodEntries, filterTasksBeforeNow, nowUtcMs]);

  const [sourceEditorOpen, setSourceEditorOpen] = useState(false);
  const [sourceEditor, setSourceEditor] = useState<ExternalCalendarSource | null>(null);
  const [sourceEditorError, setSourceEditorError] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<ExternalCalendarSource | null>(null);
  const [cacheDialogOpen, setCacheDialogOpen] = useState(false);
  const [cacheEntries, setCacheEntries] = useState<ExternalCalendarCacheEntry[]>([]);
  const [cacheSelection, setCacheSelection] = useState<string>("");
  const [cacheBusy, setCacheBusy] = useState(false);
  const importInputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    const id = window.setInterval(() => {
      setNowUtcMs(Date.now());
    }, 30_000);
    return () => window.clearInterval(id);
  }, []);

  const openAddSource = () => {
    setSourceEditor(openNewExternalCalendar());
    setSourceEditorError(null);
    setSourceEditorOpen(true);
  };

  const openEditSource = (source: ExternalCalendarSource) => {
    setSourceEditor({ ...source });
    setSourceEditorError(null);
    setSourceEditorOpen(true);
  };

  const saveSource = () => {
    if (!sourceEditor) {
      return;
    }
    if (!sourceEditor.name.trim()) {
      setSourceEditorError("Calendar name is required.");
      return;
    }
    if (!sourceEditor.location.trim()) {
      setSourceEditorError("Calendar location is required.");
      return;
    }
    saveExternalCalendarSource(sourceEditor);
    setSourceEditorOpen(false);
    setSourceEditor(null);
  };

  const renderYearView = () => {
    const year = focus.getUTCFullYear();
    return (
      <div className="calendar-period-grid year">
        {Array.from({ length: 12 }).map((_, monthIndex) => {
          const month = monthIndex + 1;
          const monthStart = firstDayOfMonth(year, month);
          const monthEntries = allDueEntries.filter(
            (entry) => entry.dueLocal.year === year && entry.dueLocal.month === month
          );
          const monthMarkers = markerEntries
            .filter((entry) => entry.dueLocal.year === year && entry.dueLocal.month === month)
            .map((entry) => entry.marker);
          const isCurrentMonth = year === nowLocal.year && month === nowLocal.month;
          const isPastMonth = monthStart.getTime() < todayMonthStart.getTime();
          return (
            <button
              key={month}
              type="button"
              className={`calendar-period-card ${isCurrentMonth ? "calendar-current-month" : ""} ${deEmphasizePastPeriods && isPastMonth ? "calendar-past-muted" : ""}`}
              onClick={() => navigateCalendar(calendarDateToIso(monthStart), "month")}
            >
              <div className="calendar-period-title">
                {monthStart.toLocaleString("en-US", { month: "long", timeZone: "UTC" })}
              </div>
              <div className="calendar-period-count">{monthEntries.length} tasks</div>
              <MarkerDots markers={monthMarkers} limit={config.policies.red_dot_limit} />
            </button>
          );
        })}
      </div>
    );
  };

  const renderQuarterView = () => {
    const months = quarterMonths(focus);
    const year = focus.getUTCFullYear();
    return (
      <div className="calendar-period-grid quarter">
        {months.map((month) => {
          const monthStart = firstDayOfMonth(year, month);
          const monthEntries = allDueEntries.filter(
            (entry) => entry.dueLocal.year === year && entry.dueLocal.month === month
          );
          const monthMarkers = markerEntries
            .filter((entry) => entry.dueLocal.year === year && entry.dueLocal.month === month)
            .map((entry) => entry.marker);
          const isCurrentMonth = year === nowLocal.year && month === nowLocal.month;
          const isPastMonth = monthStart.getTime() < todayMonthStart.getTime();
          return (
            <button
              key={month}
              type="button"
              className={`calendar-period-card ${isCurrentMonth ? "calendar-current-month" : ""} ${deEmphasizePastPeriods && isPastMonth ? "calendar-past-muted" : ""}`}
              onClick={() => navigateCalendar(calendarDateToIso(monthStart), "month")}
            >
              <div className="calendar-period-title">
                {monthStart.toLocaleString("en-US", { month: "long", timeZone: "UTC" })}
              </div>
              <div className="calendar-period-count">{monthEntries.length} tasks</div>
              <MarkerDots markers={monthMarkers} limit={config.policies.red_dot_limit} />
            </button>
          );
        })}
      </div>
    );
  };

  const renderMonthView = () => {
    const gridStart = calendarMonthGridStart(focus, config.policies.week_start);
    const weekStarts = monthWeekStarts(focus, config.policies.week_start);
    return (
      <Stack spacing={1.25}>
        <div className="calendar-weekday-row">
          {weekdayLabels(config.policies.week_start).map((label) => (
            <div key={label} className="calendar-weekday">
              {label}
            </div>
          ))}
        </div>
        <div className="calendar-month-grid">
          {Array.from({ length: 42 }).map((_, offset) => {
            const day = addDays(gridStart, offset);
            const markers = markersForDate(markerEntries, day);
            const outside = day.getUTCMonth() !== focus.getUTCMonth();
            const isCurrentDay = day.getUTCFullYear() === todayLocal.getUTCFullYear()
              && day.getUTCMonth() === todayLocal.getUTCMonth()
              && day.getUTCDate() === todayLocal.getUTCDate();
            const isPastDay = day.getTime() < todayLocal.getTime();
            return (
              <button
                key={calendarDateToIso(day)}
                type="button"
                className={`calendar-day-cell ${outside ? "outside" : ""} ${markers.length > 0 ? "has-tasks" : ""} ${isCurrentDay ? "calendar-current-day" : ""} ${deEmphasizePastPeriods && isPastDay ? "calendar-past-muted" : ""}`}
                onClick={() => navigateCalendar(calendarDateToIso(day), "day")}
              >
                <div className="calendar-day-label">{day.getUTCDate()}</div>
                <MarkerDots markers={markers} limit={config.policies.red_dot_limit} />
              </button>
            );
          })}
        </div>
        <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
          {weekStarts.map((weekStartDay) => {
            const weekEndDay = addDays(weekStartDay, 6);
            return (
              <Button
                key={calendarDateToIso(weekStartDay)}
                variant="outlined"
                size="small"
                onClick={() => navigateCalendar(calendarDateToIso(weekStartDay), "week")}
              >
                {weekStartDay.toLocaleString("en-US", { month: "short", day: "2-digit", timeZone: "UTC" })} -{" "}
                {weekEndDay.toLocaleString("en-US", { month: "short", day: "2-digit", timeZone: "UTC" })}
              </Button>
            );
          })}
        </Stack>
      </Stack>
    );
  };

  const renderWeekView = () => {
    const start = startOfWeek(focus, config.policies.week_start);
    return (
      <div className="calendar-period-grid week">
        {Array.from({ length: 7 }).map((_, offset) => {
          const day = addDays(start, offset);
          const dayEntries = entriesForDate(allDueEntries, day);
          const dayMarkers = entriesForDate(markerEntries, day).map((entry) => entry.marker);
          const isCurrentDay = day.getUTCFullYear() === todayLocal.getUTCFullYear()
            && day.getUTCMonth() === todayLocal.getUTCMonth()
            && day.getUTCDate() === todayLocal.getUTCDate();
          const isPastDay = day.getTime() < todayLocal.getTime();
          return (
            <button
              key={calendarDateToIso(day)}
              type="button"
              className={`calendar-week-card ${dayEntries.length > 0 ? "has-tasks" : ""} ${isCurrentDay ? "calendar-current-day" : ""} ${deEmphasizePastPeriods && isPastDay ? "calendar-past-muted" : ""}`}
              onClick={() => navigateCalendar(calendarDateToIso(day), "day")}
            >
              <div className="calendar-week-card-head">
                <span>{day.toLocaleString("en-US", { weekday: "short", day: "2-digit", timeZone: "UTC" })}</span>
                <span className="calendar-period-count">{dayEntries.length}</span>
              </div>
              <MarkerDots markers={dayMarkers} limit={config.policies.red_dot_limit} />
              <Stack spacing={0.5}>
                {dayEntries.slice(0, 5).map((entry) => (
                  <Typography key={`${entry.task.uuid}-${entry.dueUtcMs}`} variant="caption" className="truncate text-left">
                    {entry.task.title}
                  </Typography>
                ))}
                {dayEntries.length > 5 ? (
                  <Typography variant="caption" color="text.secondary" className="text-left">
                    +{dayEntries.length - 5} more
                  </Typography>
                ) : null}
              </Stack>
            </button>
          );
        })}
      </div>
    );
  };

  const renderDayView = () => {
    const dayEntries = entriesForDate(allDueEntries, focus).sort((a, b) => a.dueUtcMs - b.dueUtcMs);
    const dayMarkerEntries = entriesForDate(markerEntries, focus);
    const isFocusToday = focus.getUTCFullYear() === todayLocal.getUTCFullYear()
      && focus.getUTCMonth() === todayLocal.getUTCMonth()
      && focus.getUTCDate() === todayLocal.getUTCDate();
    const focusBeforeToday = focus.getTime() < todayLocal.getTime();
    const hourStart = config.day_view.hour_start;
    const hourEnd = config.day_view.hour_end;
    const nowHourFloat = nowLocal.hour + nowLocal.minute / 60;
    const rawOffset = (nowHourFloat - hourStart) / (hourEnd - hourStart + 1);
    const nowLineOffset = Math.max(0, Math.min(1, rawOffset)) * 100;
    return (
      <div className="calendar-day-view">
        <div className="calendar-day-hours">
          {isFocusToday && nowHourFloat >= hourStart && nowHourFloat <= hourEnd + 1 ? (
            <div className="calendar-now-line" style={{ top: `${nowLineOffset}%` }} />
          ) : null}
          {Array.from({ length: config.day_view.hour_end - config.day_view.hour_start + 1 }).map((_, offset) => {
            const hour = config.day_view.hour_start + offset;
            const markers = dayMarkerEntries.filter((entry) => entry.dueLocal.hour === hour).map((entry) => entry.marker);
            const pastHour = deEmphasizePastPeriods && (focusBeforeToday || (isFocusToday && hour < nowLocal.hour));
            return (
              <div key={hour} className={`calendar-hour-row ${pastHour ? "calendar-past-muted" : ""}`}>
                <span className="calendar-hour-label">{String(hour).padStart(2, "0")}:00</span>
                <MarkerDots markers={markers} limit={config.policies.red_dot_limit} />
              </div>
            );
          })}
        </div>
        <div className="calendar-day-list">
          {dayEntries.length === 0 ? (
            <Typography variant="body2" color="text.secondary">
              No tasks due on this day.
            </Typography>
          ) : (
            dayEntries.map((entry) => (
              <Paper key={`${entry.task.uuid}-${entry.dueUtcMs}`} variant="outlined" className="p-2">
                <Stack spacing={0.75}>
                  <Typography variant="subtitle2">{entry.task.title}</Typography>
                  <Typography variant="caption" color="text.secondary">
                    {formatDueDateTime(entry.dueUtcMs, config.timezone)}
                  </Typography>
                  <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap>
                    {entry.task.tags.slice(0, 5).map((tag) => (
                      <TagChip key={`${entry.task.uuid}-${tag}`} tag={tag} size="small" />
                    ))}
                  </Stack>
                </Stack>
              </Paper>
            ))
          )}
        </div>
      </div>
    );
  };

  let calendarBody = renderDayView();
  if (calendarView === "year") {
    calendarBody = renderYearView();
  } else if (calendarView === "quarter") {
    calendarBody = renderQuarterView();
  } else if (calendarView === "month") {
    calendarBody = renderMonthView();
  } else if (calendarView === "week") {
    calendarBody = renderWeekView();
  }

  return (
    <div className="grid h-full min-h-0 grid-cols-[280px_minmax(0,1fr)_340px] gap-3 p-3">
      <Paper className="min-h-0 p-3">
        <Stack spacing={1.5} className="h-full min-h-0">
          <Typography variant="h6">Calendar Views</Typography>
          <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
            {(["year", "quarter", "month", "week", "day"] as CalendarViewMode[]).map((view) => (
              <Button
                key={view}
                size="small"
                variant={calendarView === view ? "contained" : "outlined"}
                onClick={() => setCalendarView(view)}
              >
                {view[0]?.toUpperCase() + view.slice(1)}
              </Button>
            ))}
          </Stack>
          <Stack direction="row" spacing={1}>
            <Button size="small" variant="outlined" onClick={() => shiftCalendarFocus(-1)}>
              Prev
            </Button>
            <Button
              size="small"
              variant="outlined"
              onClick={() => navigateCalendar(calendarDateToIso(toCalendarDate(
                today.year,
                today.month,
                today.day
              )), calendarView)}
            >
              Today
            </Button>
            <Button size="small" variant="outlined" onClick={() => shiftCalendarFocus(1)}>
              Next
            </Button>
          </Stack>
          <Typography variant="caption" color="text.secondary">
            timezone: {config.timezone}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            focus date: {calendarDateToIso(focus)}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            due tasks: {allDueEntries.length}
          </Typography>
          <FormControlLabel
            control={(
              <Checkbox
                checked={deEmphasizePastPeriods}
                onChange={(event) => {
                  const checked = event.target.checked;
                  setDeEmphasizePastPeriods(checked);
                  setCalendarConfigToggle("de_emphasize_past_periods", checked);
                }}
              />
            )}
            label="De-emphasize past periods"
          />
          <FormControlLabel
            control={(
              <Checkbox
                checked={hidePastMarkers}
                onChange={(event) => {
                  const checked = event.target.checked;
                  setHidePastMarkers(checked);
                  setCalendarConfigToggle("hide_past_markers", checked);
                }}
              />
            )}
            label="Hide past task markers"
          />

          <Stack spacing={0.7}>
            <Typography variant="caption" color="text.secondary">Marker legend</Typography>
            <div className="calendar-legend-row"><span className="calendar-marker triangle" style={{ ["--marker-color" as string]: "var(--mui-palette-primary-main)" }} /> Kanban board task</div>
            <div className="calendar-legend-row"><span className="calendar-marker circle" style={{ ["--marker-color" as string]: "#d64545" }} /> External calendar task</div>
            <div className="calendar-legend-row"><span className="calendar-marker square" style={{ ["--marker-color" as string]: "#7f8691" }} /> Unassigned task</div>
          </Stack>

          <Box className="min-h-0 overflow-y-auto pr-1">
            <Stack spacing={1.2}>
              <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                <Button size="small" variant="outlined" startIcon={<AddIcon fontSize="small" />} onClick={openAddSource}>
                  Add Source
                </Button>
                <Button size="small" variant="outlined" startIcon={<SyncIcon fontSize="small" />} onClick={() => void syncAllExternalCalendars()} disabled={externalBusy}>
                  {externalBusy ? "Syncing..." : "Sync Enabled"}
                </Button>
                <Button
                  size="small"
                  variant="outlined"
                  onClick={() => importInputRef.current?.click()}
                >
                  Import ICS File
                </Button>
                <Button
                  size="small"
                  variant="outlined"
                  onClick={() => {
                    setCacheBusy(true);
                    void (async () => {
                      const cached = await listExternalCalendarCachedEntries();
                      setCacheEntries(cached);
                      setCacheSelection(cached[0]?.cache_id ?? "");
                      setCacheDialogOpen(true);
                      setCacheBusy(false);
                    })();
                  }}
                >
                  Import Cached ICS
                </Button>
                <input
                  ref={importInputRef}
                  type="file"
                  accept=".ics,text/calendar"
                  className="hidden"
                  onChange={(event: ChangeEvent<HTMLInputElement>) => {
                    const file = event.target.files?.[0];
                    if (file) {
                      void importExternalCalendarFile(file);
                    }
                    event.target.value = "";
                  }}
                />
              </Stack>
              {externalLastSync ? (
                <Typography variant="caption" color="text.secondary">
                  {externalLastSync}
                </Typography>
              ) : null}
              {externalCalendars.length === 0 ? (
                <Typography variant="body2" color="text.secondary">
                  No external calendar sources configured.
                </Typography>
              ) : (
                externalCalendars.map((source) => (
                  <ExternalCalendarCard
                    key={source.id}
                    source={source}
                    busy={externalBusy}
                    onSync={() => void syncExternalCalendarSource(source.id)}
                    onEdit={() => openEditSource(source)}
                    onDelete={() => setDeleteTarget(source)}
                  />
                ))
              )}
            </Stack>
          </Box>
        </Stack>
      </Paper>

      <Paper className="min-h-0 overflow-hidden p-3">
        <Stack spacing={1.25} className="h-full min-h-0">
          <Typography variant="h6">{title}</Typography>
          <div className="min-h-0 flex-1 overflow-auto pr-1">
            {calendarBody}
          </div>
        </Stack>
      </Paper>

      <Stack spacing={2} className="min-h-0">
        <Paper className="p-4">
          <Stack spacing={0.8}>
            <Typography variant="h6">Calendar Stats</Typography>
            <Typography variant="body2">period tasks: {stats.total}</Typography>
            <Typography variant="body2">pending: {stats.pending}</Typography>
            <Typography variant="body2">waiting: {stats.waiting}</Typography>
            <Typography variant="body2">completed: {stats.completed}</Typography>
            <Typography variant="body2">deleted: {stats.deleted}</Typography>
          </Stack>
        </Paper>

        <Paper className="min-h-0 p-4">
          <Stack spacing={1.2} className="h-full min-h-0">
            <Typography variant="h6">Tasks In Current Period</Typography>
            <TextField
              select
              label="Calendar"
              size="small"
              value={calendarTaskFilter}
              onChange={(event) => setCalendarTaskFilter(event.target.value)}
            >
              <MenuItem value="__all__">All calendars and tasks</MenuItem>
              <MenuItem value="__none__">No calendar source</MenuItem>
              {calendarFilterOptions.map((entry) => (
                <MenuItem key={entry.id} value={entry.id}>
                  {entry.label}
                </MenuItem>
              ))}
            </TextField>
            <FormControlLabel
              control={(
                <Checkbox
                  checked={filterTasksBeforeNow}
                  onChange={(event) => {
                    const checked = event.target.checked;
                    setFilterTasksBeforeNow(checked);
                    setCalendarConfigToggle("filter_tasks_before_now", checked);
                  }}
                />
              )}
              label="Filter out tasks before now"
            />
            <div className="min-h-0 flex-1 overflow-y-auto pr-1">
              <Stack spacing={1}>
                {visiblePeriodEntries.length === 0 ? (
                  <Typography variant="body2" color="text.secondary">
                    No tasks due in this calendar period for the selected filters.
                  </Typography>
                ) : (
                  visiblePeriodEntries.map((entry) => (
                    <Paper key={`${entry.task.uuid}-${entry.dueUtcMs}`} variant="outlined" className="p-2">
                      <Stack spacing={0.75}>
                        <Typography variant="subtitle2">{entry.task.title}</Typography>
                        <Typography variant="caption" color="text.secondary">
                          {formatDueDateTime(entry.dueUtcMs, config.timezone)}
                        </Typography>
                        {entry.task.project ? (
                          <Typography variant="caption" className="rounded-md border border-current/15 px-1.5 py-0.5">
                            project:{entry.task.project}
                          </Typography>
                        ) : null}
                        <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap>
                          {entry.task.tags.slice(0, 4).map((tag) => (
                            <TagChip key={`${entry.task.uuid}-${tag}`} tag={tag} size="small" />
                          ))}
                        </Stack>
                      </Stack>
                    </Paper>
                  ))
                )}
              </Stack>
            </div>
          </Stack>
        </Paper>

        {error ? <Alert severity="error">{error}</Alert> : null}
      </Stack>

      <Dialog open={sourceEditorOpen} onClose={() => setSourceEditorOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{sourceEditor ? (externalCalendars.some((entry) => entry.id === sourceEditor.id) ? "Edit External Calendar" : "Add External Calendar") : "External Calendar"}</DialogTitle>
        <DialogContent dividers>
          {sourceEditor ? (
            <Stack spacing={1.5}>
              {sourceEditorError ? <Alert severity="error">{sourceEditorError}</Alert> : null}
              <FormControlLabel
                control={(
                  <Checkbox
                    checked={sourceEditor.enabled}
                    onChange={(event) => setSourceEditor((prev) => (prev ? { ...prev, enabled: event.target.checked } : prev))}
                    disabled={sourceEditor.imported_ics_file}
                  />
                )}
                label="Enable This Calendar"
              />
              {sourceEditor.imported_ics_file ? (
                <Typography variant="caption" color="text.secondary">
                  Imported ICS calendars are local snapshots. Use Import ICS File again to refresh data.
                </Typography>
              ) : null}
              <TextField
                label="Calendar Name"
                value={sourceEditor.name}
                onChange={(event) => setSourceEditor((prev) => (prev ? { ...prev, name: event.target.value } : prev))}
              />
              <TextField
                label="Color"
                type="color"
                value={sourceEditor.color}
                onChange={(event) => setSourceEditor((prev) => (prev ? { ...prev, color: event.target.value } : prev))}
              />
              <TextField
                label="Location (ICS or webcal URL)"
                value={sourceEditor.location}
                placeholder="webcal://example.com/calendar.ics"
                disabled={sourceEditor.imported_ics_file}
                onChange={(event) => setSourceEditor((prev) => (prev ? { ...prev, location: event.target.value } : prev))}
              />
              <TextField
                select
                label="Refresh Calendar"
                value={String(sourceEditor.refresh_minutes)}
                disabled={sourceEditor.imported_ics_file}
                onChange={(event) => setSourceEditor((prev) => (
                  prev ? { ...prev, refresh_minutes: Number(event.target.value) || 0 } : prev
                ))}
              >
                <MenuItem value="0">Disabled (manual only)</MenuItem>
                <MenuItem value="5">Every 5 minutes</MenuItem>
                <MenuItem value="15">Every 15 minutes</MenuItem>
                <MenuItem value="30">Every 30 minutes</MenuItem>
                <MenuItem value="60">Every 60 minutes</MenuItem>
                <MenuItem value="360">Every 6 hours</MenuItem>
                <MenuItem value="1440">Every 24 hours</MenuItem>
              </TextField>
              <FormControlLabel
                control={(
                  <Checkbox
                    checked={sourceEditor.read_only}
                    onChange={(event) => setSourceEditor((prev) => (prev ? { ...prev, read_only: event.target.checked } : prev))}
                  />
                )}
                label="Read Only"
              />
              <FormControlLabel
                control={(
                  <Checkbox
                    checked={sourceEditor.show_reminders}
                    onChange={(event) => setSourceEditor((prev) => (prev ? { ...prev, show_reminders: event.target.checked } : prev))}
                  />
                )}
                label="Show Reminders"
              />
              <FormControlLabel
                control={(
                  <Checkbox
                    checked={sourceEditor.offline_support}
                    onChange={(event) => setSourceEditor((prev) => (prev ? { ...prev, offline_support: event.target.checked } : prev))}
                  />
                )}
                label="Offline Support"
              />
            </Stack>
          ) : null}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setSourceEditorOpen(false)} disabled={externalBusy}>
            Cancel
          </Button>
          <Button onClick={saveSource} variant="contained" disabled={externalBusy}>
            {externalBusy ? "Saving..." : "Save"}
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog open={Boolean(deleteTarget)} onClose={() => setDeleteTarget(null)} maxWidth="xs" fullWidth>
        <DialogTitle>Delete External Calendar</DialogTitle>
        <DialogContent dividers>
          <Typography variant="body2">
            Delete external calendar &apos;{deleteTarget?.name ?? ""}&apos;?
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteTarget(null)}>Cancel</Button>
          <Button
            color="error"
            variant="contained"
            onClick={() => {
              if (!deleteTarget) {
                return;
              }
              deleteExternalCalendarSource(deleteTarget.id);
              setDeleteTarget(null);
            }}
          >
            Delete
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog open={cacheDialogOpen} onClose={() => setCacheDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Import Cached ICS</DialogTitle>
        <DialogContent dividers>
          <Stack spacing={1.25}>
            {cacheEntries.length === 0 ? (
              <Typography variant="body2" color="text.secondary">
                No cached ICS snapshots found.
              </Typography>
            ) : (
              <>
                <TextField
                  select
                  label="Cached snapshot"
                  value={cacheSelection}
                  onChange={(event) => setCacheSelection(event.target.value)}
                >
                  {cacheEntries.map((entry) => (
                    <MenuItem key={entry.cache_id} value={entry.cache_id}>
                      {entry.name} ({entry.kind}) - {entry.cached_at}
                    </MenuItem>
                  ))}
                </TextField>
                {cacheEntries.find((entry) => entry.cache_id === cacheSelection) ? (
                  <Typography variant="caption" color="text.secondary">
                    {cacheEntries.find((entry) => entry.cache_id === cacheSelection)?.location}
                  </Typography>
                ) : null}
              </>
            )}
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCacheDialogOpen(false)} disabled={externalBusy || cacheBusy}>
            Cancel
          </Button>
          <Button
            variant="contained"
            disabled={cacheEntries.length === 0 || !cacheSelection || externalBusy || cacheBusy}
            onClick={() => {
              const selected = cacheEntries.find((entry) => entry.cache_id === cacheSelection);
              if (!selected) {
                return;
              }
              setCacheBusy(true);
              void (async () => {
                await importExternalCalendarFromCache(selected);
                setCacheBusy(false);
                setCacheDialogOpen(false);
              })();
            }}
          >
            Import
          </Button>
        </DialogActions>
      </Dialog>
    </div>
  );
}

function nowDateTime(timezone: string, nowUtcMs: number) {
  const formatter = new Intl.DateTimeFormat("en-US", {
    timeZone: timezone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false
  });
  const parts = formatter.formatToParts(new Date(nowUtcMs));
  let year = 1970;
  let month = 1;
  let day = 1;
  let hour = 0;
  let minute = 0;
  for (const part of parts) {
    if (part.type === "year") {
      year = Number(part.value);
    } else if (part.type === "month") {
      month = Number(part.value);
    } else if (part.type === "day") {
      day = Number(part.value);
    } else if (part.type === "hour") {
      hour = Number(part.value);
    } else if (part.type === "minute") {
      minute = Number(part.value);
    }
  }
  return { year, month, day, hour, minute };
}
