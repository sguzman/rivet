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
    features?: {
      contacts?: boolean;
      dictionary?: boolean;
      map?: boolean;
    };
  };
  map?: {
    enabled?: boolean;
    martin_base_url?: string;
    default_source?: string;
    default_center?: number[];
    default_zoom?: number;
    min_zoom?: number;
    max_zoom?: number;
    max_parallel_image_requests?: number;
    cancel_pending_tile_requests_while_zooming?: boolean;
    hide_when_unavailable?: boolean;
  };
  dictionary?: {
    enabled?: boolean;
    default_language?: string;
    max_results?: number;
    search_mode?: "exact" | "prefix" | "fuzzy" | "fts" | string;
    hide_when_unavailable?: boolean;
    postgres?: {
      host?: string;
      port?: number;
      user?: string;
      password?: string;
      database?: string;
      schema?: string;
      sslmode?: "disable" | string;
      connect_timeout_secs?: number;
      max_connection_retries?: number;
      retry_backoff_ms?: number;
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
