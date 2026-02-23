import { z } from "zod";

export const TaskStatusSchema = z.enum(["Pending", "Completed", "Deleted", "Waiting"]);
export const TaskPrioritySchema = z.enum(["Low", "Medium", "High"]);

export const TaskDtoSchema = z.object({
  uuid: z.string().min(1),
  id: z.number().int().nonnegative().nullable(),
  title: z.string(),
  description: z.string(),
  status: TaskStatusSchema,
  project: z.string().nullable(),
  tags: z.array(z.string()),
  priority: TaskPrioritySchema.nullable(),
  due: z.string().nullable(),
  wait: z.string().nullable(),
  scheduled: z.string().nullable(),
  created: z.string().nullable(),
  modified: z.string().nullable()
});

export const TaskDtoArraySchema = z.array(TaskDtoSchema);

export const TaskCreateSchema = z.object({
  title: z.string().min(1),
  description: z.string(),
  project: z.string().nullable(),
  tags: z.array(z.string()),
  priority: TaskPrioritySchema.nullable(),
  due: z.string().nullable(),
  wait: z.string().nullable(),
  scheduled: z.string().nullable()
});

export const TaskPatchSchema = z.object({
  title: z.string().optional(),
  description: z.string().optional(),
  project: z.string().nullable().optional(),
  tags: z.array(z.string()).optional(),
  priority: TaskPrioritySchema.nullable().optional(),
  due: z.string().nullable().optional(),
  wait: z.string().nullable().optional(),
  scheduled: z.string().nullable().optional()
});

export const TaskUpdateArgsSchema = z.object({
  uuid: z.string().min(1),
  patch: TaskPatchSchema
});

export const ExternalCalendarSourceSchema = z.object({
  id: z.string().min(1),
  name: z.string(),
  color: z.string(),
  location: z.string(),
  refresh_minutes: z.number().int().min(0),
  enabled: z.boolean(),
  imported_ics_file: z.boolean(),
  read_only: z.boolean(),
  show_reminders: z.boolean(),
  offline_support: z.boolean()
});

export const ExternalCalendarSyncResultSchema = z.object({
  calendar_id: z.string(),
  created: z.number().int().min(0),
  updated: z.number().int().min(0),
  deleted: z.number().int().min(0),
  remote_events: z.number().int().min(0),
  refresh_minutes: z.number().int().min(0)
});

export const TagKeySchema = z.object({
  id: z.string(),
  label: z.string().optional(),
  selection: z.string().optional(),
  color: z.string().optional(),
  allow_custom_values: z.boolean().optional(),
  values: z.array(z.string()).optional()
});

export const TagSchemaSchema = z.object({
  version: z.number().int().optional(),
  keys: z.array(TagKeySchema).optional()
}).passthrough();

export const RivetRuntimeConfigSchema = z.object({
  version: z.number().int().optional(),
  mode: z.string().optional(),
  timezone: z.string().optional(),
  app: z.object({
    mode: z.string().optional()
  }).passthrough().optional(),
  logging: z.object({
    directory: z.string().optional(),
    file_prefix: z.string().optional()
  }).passthrough().optional(),
  time: z.object({
    timezone: z.string().optional()
  }).passthrough().optional(),
  calendar: z.object({
    version: z.number().int().optional(),
    timezone: z.string().optional(),
    policies: z.object({
      week_start: z.string().optional(),
      red_dot_limit: z.number().int().optional(),
      task_list_limit: z.number().int().optional(),
      task_list_window_days: z.number().int().optional()
    }).passthrough().optional(),
    visibility: z.object({
      pending: z.boolean().optional(),
      waiting: z.boolean().optional(),
      completed: z.boolean().optional(),
      deleted: z.boolean().optional()
    }).passthrough().optional(),
    day_view: z.object({
      hour_start: z.number().int().optional(),
      hour_end: z.number().int().optional()
    }).passthrough().optional(),
    toggles: z.object({
      de_emphasize_past_periods: z.boolean().optional(),
      filter_tasks_before_now: z.boolean().optional(),
      hide_past_markers: z.boolean().optional()
    }).passthrough().optional()
  }).passthrough().optional()
}).passthrough();

export function describeSchemaError(prefix: string, error: z.ZodError): string {
  const firstIssue = error.issues[0];
  if (!firstIssue) {
    return `${prefix}: schema validation failed`;
  }
  const path = firstIssue.path.length > 0 ? firstIssue.path.join(".") : "(root)";
  return `${prefix}: ${path} ${firstIssue.message}`;
}
