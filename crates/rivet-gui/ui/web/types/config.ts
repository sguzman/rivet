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
  };
}
