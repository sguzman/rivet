export interface TagSchema {
  version?: number;
  keys?: TagKey[];
}

export interface TagKey {
  id: string;
  label?: string;
  selection?: "single" | "multi" | string;
  color?: string;
  allow_custom_values?: boolean;
  values?: string[];
}

export interface RivetRuntimeConfig {
  version?: number;
  mode?: "dev" | "prod" | string;
  timezone?: string;
  app?: {
    mode?: "dev" | "prod" | string;
  };
  logging?: {
    directory?: string;
    file_prefix?: string;
  };
  time?: {
    timezone?: string;
  };
  notifications?: {
    due?: {
      enabled?: boolean;
      pre_notify_enabled?: boolean;
      pre_notify_minutes?: number;
      scan_interval_seconds?: number;
    };
  };
  ui?: {
    default_theme?: "day" | "night" | string;
    theme?: {
      mode?: "day" | "night" | string;
      follow_system?: boolean;
    };
  };
  calendar?: {
    version?: number;
    timezone?: string;
    policies?: {
      week_start?: "monday" | "sunday" | string;
      red_dot_limit?: number;
      task_list_limit?: number;
      task_list_window_days?: number;
    };
    visibility?: {
      pending?: boolean;
      waiting?: boolean;
      completed?: boolean;
      deleted?: boolean;
    };
    day_view?: {
      hour_start?: number;
      hour_end?: number;
    };
    toggles?: {
      de_emphasize_past_periods?: boolean;
      filter_tasks_before_now?: boolean;
      hide_past_markers?: boolean;
    };
  };
}
