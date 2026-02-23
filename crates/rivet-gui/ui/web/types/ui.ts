import type { ExternalCalendarSource, TaskDto, TaskStatus } from "./core";

export type WorkspaceTab = "tasks" | "kanban" | "calendar";
export type ThemeMode = "day" | "night";
export type StatusFilter = "all" | TaskStatus;
export type PriorityFilter = "all" | "low" | "medium" | "high" | "none";
export type DueFilter = "all" | "has_due" | "no_due";

export type CalendarViewMode = "year" | "quarter" | "month" | "week" | "day";
export type CalendarWeekStart = "monday" | "sunday";
export type CalendarMarkerShape = "triangle" | "circle" | "square";

export interface TaskFilters {
  search: string;
  status: StatusFilter;
  project: string;
  tag: string;
  priority: PriorityFilter;
  due: DueFilter;
}

export interface KanbanBoardDef {
  id: string;
  name: string;
  color: string;
}

export interface AddTaskDialogContext {
  boardId: string | null;
  lockBoardSelection: boolean;
  allowRecurrence: boolean;
}

export interface RecurrenceDraft {
  pattern: "none" | "daily" | "weekly" | "months" | "monthly" | "yearly";
  time: string;
  days: string[];
  months: string[];
  monthDay: string;
}

export interface CalendarPolicies {
  week_start: CalendarWeekStart | string;
  red_dot_limit: number;
  task_list_limit: number;
  task_list_window_days: number;
}

export interface CalendarVisibility {
  pending: boolean;
  waiting: boolean;
  completed: boolean;
  deleted: boolean;
}

export interface CalendarDayView {
  hour_start: number;
  hour_end: number;
}

export interface CalendarToggles {
  de_emphasize_past_periods: boolean;
  filter_tasks_before_now: boolean;
  hide_past_markers: boolean;
}

export interface EffectiveCalendarConfig {
  timezone: string;
  policies: CalendarPolicies;
  visibility: CalendarVisibility;
  day_view: CalendarDayView;
  toggles: CalendarToggles;
}

export interface CalendarTaskMarker {
  shape: CalendarMarkerShape;
  color: string;
}

export interface ZonedDateTimeParts {
  year: number;
  month: number;
  day: number;
  weekday: number;
  hour: number;
  minute: number;
  second: number;
}

export interface CalendarDueTaskEntry {
  task: TaskDto;
  dueUtcMs: number;
  dueLocal: ZonedDateTimeParts;
  marker: CalendarTaskMarker;
}

export interface CalendarStats {
  total: number;
  pending: number;
  waiting: number;
  completed: number;
  deleted: number;
}

export interface ExternalCalendarState {
  sources: ExternalCalendarSource[];
  busy: boolean;
  lastSyncMessage: string | null;
}

export interface DueNotificationConfig {
  enabled: boolean;
  pre_notify_enabled: boolean;
  pre_notify_minutes: number;
}
