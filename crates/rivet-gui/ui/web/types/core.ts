export type TaskStatus = "Pending" | "Completed" | "Deleted" | "Waiting";
export type TaskPriority = "Low" | "Medium" | "High";

export interface TaskDto {
  uuid: string;
  id: number | null;
  title: string;
  description: string;
  status: TaskStatus;
  project: string | null;
  tags: string[];
  priority: TaskPriority | null;
  due: string | null;
  wait: string | null;
  scheduled: string | null;
  created: string | null;
  modified: string | null;
}

export interface TasksListArgs {
  query: string | null;
  status: TaskStatus | null;
  project: string | null;
  tag: string | null;
}

export interface TaskCreate {
  title: string;
  description: string;
  project: string | null;
  tags: string[];
  priority: TaskPriority | null;
  due: string | null;
  wait: string | null;
  scheduled: string | null;
}

export interface TaskPatch {
  title?: string;
  description?: string;
  project?: string | null;
  tags?: string[];
  priority?: TaskPriority | null;
  due?: string | null;
  wait?: string | null;
  scheduled?: string | null;
}

export interface TaskUpdateArgs {
  uuid: string;
  patch: TaskPatch;
}

export interface TaskIdArg {
  uuid: string;
}

export interface ExternalCalendarSource {
  id: string;
  name: string;
  color: string;
  location: string;
  refresh_minutes: number;
  enabled: boolean;
  imported_ics_file: boolean;
  read_only: boolean;
  show_reminders: boolean;
  offline_support: boolean;
}

export interface ExternalCalendarSyncResult {
  calendar_id: string;
  created: number;
  updated: number;
  deleted: number;
  remote_events: number;
  refresh_minutes: number;
}

export interface ExternalCalendarCacheEntry {
  cache_id: string;
  name: string;
  location: string;
  color: string;
  cached_at: string;
  kind: string;
}
