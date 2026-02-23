import type { TaskDto, TaskStatus } from "../types/core";
import type { RivetRuntimeConfig } from "../types/config";
import type {
  CalendarDueTaskEntry,
  CalendarStats,
  CalendarTaskMarker,
  CalendarViewMode,
  CalendarWeekStart,
  EffectiveCalendarConfig,
  ZonedDateTimeParts
} from "../types/ui";
import { BOARD_TAG_KEY, CAL_COLOR_TAG_KEY, CAL_SOURCE_TAG_KEY, firstTagValue, normalizeMarkerColor } from "./tags";

const DAY_MS = 24 * 60 * 60 * 1000;
const DEFAULT_TIMEZONE = "America/Mexico_City";

export function resolveCalendarConfig(runtimeConfig: RivetRuntimeConfig | null): EffectiveCalendarConfig {
  const calendarTimezone = runtimeConfig?.calendar?.timezone?.trim();
  const fallbackTimezone = runtimeConfig?.time?.timezone?.trim() || runtimeConfig?.timezone?.trim() || DEFAULT_TIMEZONE;
  const timezoneCandidate = calendarTimezone || fallbackTimezone;
  const timezone = resolveTimezone(timezoneCandidate);

  const weekStartRaw = runtimeConfig?.calendar?.policies?.week_start ?? "monday";
  const weekStart = weekStartRaw.toLowerCase() === "sunday" ? "sunday" : "monday";
  const redDotLimit = clampPositiveInt(runtimeConfig?.calendar?.policies?.red_dot_limit, 5000);
  const taskListLimit = clampPositiveInt(runtimeConfig?.calendar?.policies?.task_list_limit, 200);
  const taskListWindowDays = clampPositiveInt(runtimeConfig?.calendar?.policies?.task_list_window_days, 365);

  const hourStart = clampHour(runtimeConfig?.calendar?.day_view?.hour_start ?? 0);
  const hourEndRaw = clampHour(runtimeConfig?.calendar?.day_view?.hour_end ?? 23);
  const hourEnd = hourEndRaw < hourStart ? hourStart : hourEndRaw;

  return {
    timezone,
    policies: {
      week_start: weekStart,
      red_dot_limit: redDotLimit,
      task_list_limit: taskListLimit,
      task_list_window_days: taskListWindowDays
    },
    visibility: {
      pending: runtimeConfig?.calendar?.visibility?.pending ?? true,
      waiting: runtimeConfig?.calendar?.visibility?.waiting ?? true,
      completed: runtimeConfig?.calendar?.visibility?.completed ?? true,
      deleted: runtimeConfig?.calendar?.visibility?.deleted ?? true
    },
    day_view: {
      hour_start: hourStart,
      hour_end: hourEnd
    },
    toggles: {
      de_emphasize_past_periods: runtimeConfig?.calendar?.toggles?.de_emphasize_past_periods ?? true,
      filter_tasks_before_now: runtimeConfig?.calendar?.toggles?.filter_tasks_before_now ?? true,
      hide_past_markers: runtimeConfig?.calendar?.toggles?.hide_past_markers ?? true
    }
  };
}

function resolveTimezone(timezoneCandidate: string): string {
  try {
    new Intl.DateTimeFormat("en-US", { timeZone: timezoneCandidate });
    return timezoneCandidate;
  } catch {
    return DEFAULT_TIMEZONE;
  }
}

function clampPositiveInt(value: number | undefined, fallback: number): number {
  if (!value || value <= 0 || !Number.isFinite(value)) {
    return fallback;
  }
  return Math.floor(value);
}

function clampHour(value: number): number {
  if (!Number.isFinite(value)) {
    return 0;
  }
  return Math.max(0, Math.min(23, Math.floor(value)));
}

export function toCalendarDate(year: number, month: number, day: number): Date {
  return new Date(Date.UTC(year, month - 1, day, 12, 0, 0, 0));
}

export function calendarDateToIso(date: Date): string {
  return `${date.getUTCFullYear()}-${String(date.getUTCMonth() + 1).padStart(2, "0")}-${String(date.getUTCDate()).padStart(2, "0")}`;
}

export function calendarDateFromIso(iso: string): Date {
  const match = iso.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  if (!match) {
    const now = new Date();
    return toCalendarDate(now.getUTCFullYear(), now.getUTCMonth() + 1, now.getUTCDate());
  }
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  return toCalendarDate(year, month, day);
}

export function addDays(date: Date, days: number): Date {
  return new Date(date.getTime() + days * DAY_MS);
}

export function firstDayOfMonth(year: number, month: number): Date {
  return toCalendarDate(year, month, 1);
}

export function lastDayOfMonth(year: number, month: number): Date {
  const nextMonth = month === 12 ? toCalendarDate(year + 1, 1, 1) : toCalendarDate(year, month + 1, 1);
  return addDays(nextMonth, -1);
}

export function shiftMonths(date: Date, months: number): Date {
  let year = date.getUTCFullYear();
  let month = date.getUTCMonth() + 1 + months;
  while (month < 1) {
    month += 12;
    year -= 1;
  }
  while (month > 12) {
    month -= 12;
    year += 1;
  }
  const day = Math.min(date.getUTCDate(), lastDayOfMonth(year, month).getUTCDate());
  return toCalendarDate(year, month, day);
}

export function shiftYears(date: Date, years: number): Date {
  const year = date.getUTCFullYear() + years;
  const month = date.getUTCMonth() + 1;
  const day = Math.min(date.getUTCDate(), lastDayOfMonth(year, month).getUTCDate());
  return toCalendarDate(year, month, day);
}

export function weekStartDay(weekStart: CalendarWeekStart | string): number {
  return weekStart.toLowerCase() === "sunday" ? 0 : 1;
}

export function startOfWeek(date: Date, weekStart: CalendarWeekStart | string): Date {
  const day = date.getUTCDay();
  const start = weekStartDay(weekStart);
  const diff = (7 + day - start) % 7;
  return addDays(date, -diff);
}

export function weekdayLabels(weekStart: CalendarWeekStart | string): string[] {
  const labels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  const start = weekStartDay(weekStart);
  return [...labels.slice(start), ...labels.slice(0, start)];
}

export function shiftCalendarFocus(current: Date, view: CalendarViewMode, step: number, weekStart: CalendarWeekStart | string): Date {
  switch (view) {
    case "year":
      return shiftYears(current, step);
    case "quarter":
      return shiftMonths(current, step * 3);
    case "month":
      return shiftMonths(current, step);
    case "week":
      return addDays(startOfWeek(current, weekStart), step * 7);
    case "day":
      return addDays(current, step);
    default:
      return current;
  }
}

export function calendarWindow(view: CalendarViewMode, focus: Date, weekStart: CalendarWeekStart | string): { start: Date; end: Date } {
  switch (view) {
    case "year":
      return {
        start: firstDayOfMonth(focus.getUTCFullYear(), 1),
        end: lastDayOfMonth(focus.getUTCFullYear(), 12)
      };
    case "quarter": {
      const quarterStartMonth = Math.floor((focus.getUTCMonth()) / 3) * 3 + 1;
      return {
        start: firstDayOfMonth(focus.getUTCFullYear(), quarterStartMonth),
        end: lastDayOfMonth(focus.getUTCFullYear(), quarterStartMonth + 2)
      };
    }
    case "month":
      return {
        start: firstDayOfMonth(focus.getUTCFullYear(), focus.getUTCMonth() + 1),
        end: lastDayOfMonth(focus.getUTCFullYear(), focus.getUTCMonth() + 1)
      };
    case "week": {
      const start = startOfWeek(focus, weekStart);
      return { start, end: addDays(start, 6) };
    }
    case "day":
      return { start: focus, end: focus };
    default:
      return { start: focus, end: focus };
  }
}

export function calendarTitleForView(view: CalendarViewMode, focus: Date, weekStart: CalendarWeekStart | string): string {
  const year = focus.getUTCFullYear();
  if (view === "year") {
    return `Year View ${year}`;
  }
  if (view === "quarter") {
    const quarter = Math.floor(focus.getUTCMonth() / 3) + 1;
    const quarterStartMonth = Math.floor(focus.getUTCMonth() / 3) * 3 + 1;
    const start = firstDayOfMonth(year, quarterStartMonth).toLocaleString("en-US", { month: "short", timeZone: "UTC" });
    const end = firstDayOfMonth(year, quarterStartMonth + 2).toLocaleString("en-US", { month: "short", timeZone: "UTC" });
    return `Quarter View Q${quarter} ${year} (${start}-${end})`;
  }
  if (view === "month") {
    return `Month View ${focus.toLocaleString("en-US", { month: "long", year: "numeric", timeZone: "UTC" })}`;
  }
  if (view === "week") {
    const start = startOfWeek(focus, weekStart);
    const end = addDays(start, 6);
    return `Week View ${calendarDateToIso(start)} - ${calendarDateToIso(end)}`;
  }
  return `Day View ${focus.toLocaleString("en-US", { weekday: "long", year: "numeric", month: "2-digit", day: "2-digit", timeZone: "UTC" })}`;
}

export function monthWeekStarts(focus: Date, weekStart: CalendarWeekStart | string): Date[] {
  const monthFirst = firstDayOfMonth(focus.getUTCFullYear(), focus.getUTCMonth() + 1);
  const monthLast = lastDayOfMonth(focus.getUTCFullYear(), focus.getUTCMonth() + 1);
  const gridStart = startOfWeek(monthFirst, weekStart);
  const values: Date[] = [];
  for (let row = 0; row < 6; row += 1) {
    const weekStartDate = addDays(gridStart, row * 7);
    const weekEndDate = addDays(weekStartDate, 6);
    if (weekEndDate < monthFirst || weekStartDate > monthLast) {
      continue;
    }
    values.push(weekStartDate);
  }
  return values;
}

export function quarterMonths(focus: Date): number[] {
  const quarterStartMonth = Math.floor(focus.getUTCMonth() / 3) * 3 + 1;
  return [quarterStartMonth, quarterStartMonth + 1, quarterStartMonth + 2];
}

export function todayInTimezone(timezone: string): Date {
  const now = zonedDateTimeParts(Date.now(), timezone);
  return toCalendarDate(now.year, now.month, now.day);
}

function statusVisible(status: TaskStatus, config: EffectiveCalendarConfig): boolean {
  switch (status) {
    case "Pending":
      return config.visibility.pending;
    case "Waiting":
      return config.visibility.waiting;
    case "Completed":
      return config.visibility.completed;
    case "Deleted":
      return config.visibility.deleted;
    default:
      return true;
  }
}

export function parseTaskDueUtcMs(rawDue: string): number | null {
  const taskwarrior = rawDue.match(/^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})Z$/);
  if (taskwarrior) {
    const year = Number(taskwarrior[1]);
    const month = Number(taskwarrior[2]);
    const day = Number(taskwarrior[3]);
    const hour = Number(taskwarrior[4]);
    const minute = Number(taskwarrior[5]);
    const second = Number(taskwarrior[6]);
    return Date.UTC(year, month - 1, day, hour, minute, second);
  }
  const parsed = Date.parse(rawDue);
  if (Number.isNaN(parsed)) {
    return null;
  }
  return parsed;
}

export function isCalendarEventTask(task: TaskDto): boolean {
  return firstTagValue(task.tags, CAL_SOURCE_TAG_KEY) !== null;
}

export function canManuallyCompleteTask(task: TaskDto, nowUtcMs: number): boolean {
  if (!isCalendarEventTask(task)) {
    return true;
  }
  const dueRaw = task.due?.trim();
  if (!dueRaw) {
    return false;
  }
  const dueUtcMs = parseTaskDueUtcMs(dueRaw);
  if (dueUtcMs === null) {
    return false;
  }
  return dueUtcMs <= nowUtcMs;
}

export function zonedDateTimeParts(utcMs: number, timezone: string): ZonedDateTimeParts {
  const formatter = new Intl.DateTimeFormat("en-US", {
    timeZone: timezone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    weekday: "short",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23"
  });
  const parts = formatter.formatToParts(new Date(utcMs));
  let year = 1970;
  let month = 1;
  let day = 1;
  let hour = 0;
  let minute = 0;
  let second = 0;
  let weekday = 0;

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
    } else if (part.type === "second") {
      second = Number(part.value);
    } else if (part.type === "weekday") {
      weekday = shortWeekdayToIndex(part.value);
    }
  }

  return { year, month, day, weekday, hour, minute, second };
}

function shortWeekdayToIndex(value: string): number {
  const normalized = value.trim().toLowerCase();
  if (normalized.startsWith("sun")) {
    return 0;
  }
  if (normalized.startsWith("mon")) {
    return 1;
  }
  if (normalized.startsWith("tue")) {
    return 2;
  }
  if (normalized.startsWith("wed")) {
    return 3;
  }
  if (normalized.startsWith("thu")) {
    return 4;
  }
  if (normalized.startsWith("fri")) {
    return 5;
  }
  return 6;
}

function markerForTask(task: TaskDto, boardColors: Record<string, string>, calendarColors: Record<string, string>): CalendarTaskMarker {
  const calendarId = firstTagValue(task.tags, CAL_SOURCE_TAG_KEY);
  if (calendarId) {
    const color = calendarColors[calendarId]
      || calendarColors[calendarId.toLowerCase()]
      || normalizeMarkerColor(firstTagValue(task.tags, CAL_COLOR_TAG_KEY) ?? "#d64545");
    return {
      shape: "circle",
      color
    };
  }

  const boardId = firstTagValue(task.tags, BOARD_TAG_KEY);
  if (boardId) {
    return {
      shape: "triangle",
      color: boardColors[boardId] || "hsl(212 74% 54%)"
    };
  }

  return {
    shape: "square",
    color: "#7f8691"
  };
}

export function collectCalendarDueTasks(
  tasks: TaskDto[],
  config: EffectiveCalendarConfig,
  boardColors: Record<string, string>,
  calendarColors: Record<string, string>
): CalendarDueTaskEntry[] {
  const entries: CalendarDueTaskEntry[] = [];
  for (const task of tasks) {
    if (!statusVisible(task.status, config)) {
      continue;
    }
    const dueRaw = task.due?.trim();
    if (!dueRaw) {
      continue;
    }
    const dueUtcMs = parseTaskDueUtcMs(dueRaw);
    if (dueUtcMs === null) {
      continue;
    }
    entries.push({
      task,
      dueUtcMs,
      dueLocal: zonedDateTimeParts(dueUtcMs, config.timezone),
      marker: markerForTask(task, boardColors, calendarColors)
    });
  }
  entries.sort((a, b) => a.dueUtcMs - b.dueUtcMs);
  return entries;
}

function dateTupleToNumber(year: number, month: number, day: number): number {
  return (year * 10000) + (month * 100) + day;
}

export function periodTasks(entries: CalendarDueTaskEntry[], view: CalendarViewMode, focus: Date, weekStart: CalendarWeekStart | string): CalendarDueTaskEntry[] {
  const window = calendarWindow(view, focus, weekStart);
  const startValue = dateTupleToNumber(window.start.getUTCFullYear(), window.start.getUTCMonth() + 1, window.start.getUTCDate());
  const endValue = dateTupleToNumber(window.end.getUTCFullYear(), window.end.getUTCMonth() + 1, window.end.getUTCDate());
  return entries.filter((entry) => {
    const dueValue = dateTupleToNumber(entry.dueLocal.year, entry.dueLocal.month, entry.dueLocal.day);
    return dueValue >= startValue && dueValue <= endValue;
  });
}

export function periodStats(entries: CalendarDueTaskEntry[]): CalendarStats {
  const stats: CalendarStats = {
    total: 0,
    pending: 0,
    waiting: 0,
    completed: 0,
    deleted: 0
  };
  for (const entry of entries) {
    stats.total += 1;
    if (entry.task.status === "Pending") {
      stats.pending += 1;
    } else if (entry.task.status === "Waiting") {
      stats.waiting += 1;
    } else if (entry.task.status === "Completed") {
      stats.completed += 1;
    } else if (entry.task.status === "Deleted") {
      stats.deleted += 1;
    }
  }
  return stats;
}

export function entriesForDate(entries: CalendarDueTaskEntry[], day: Date): CalendarDueTaskEntry[] {
  const year = day.getUTCFullYear();
  const month = day.getUTCMonth() + 1;
  const date = day.getUTCDate();
  return entries.filter((entry) => entry.dueLocal.year === year && entry.dueLocal.month === month && entry.dueLocal.day === date);
}

export function markersForDate(entries: CalendarDueTaskEntry[], day: Date): CalendarTaskMarker[] {
  return entriesForDate(entries, day).map((entry) => entry.marker);
}

export function formatDueDateTime(utcMs: number, timezone: string): string {
  const formatter = new Intl.DateTimeFormat("en-CA", {
    timeZone: timezone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: "h23"
  });
  return `${formatter.format(new Date(utcMs))} (${timezone})`;
}

export function calendarMonthGridStart(focus: Date, weekStart: CalendarWeekStart | string): Date {
  const monthFirst = firstDayOfMonth(focus.getUTCFullYear(), focus.getUTCMonth() + 1);
  return startOfWeek(monthFirst, weekStart);
}
