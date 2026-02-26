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

export const ContactFieldValueSchema = z.object({
  value: z.string(),
  kind: z.string(),
  is_primary: z.boolean()
});

export const ContactAddressSchema = z.object({
  kind: z.string(),
  street: z.string(),
  city: z.string(),
  region: z.string(),
  postal_code: z.string(),
  country: z.string()
});

export const ContactDtoSchema = z.object({
  id: z.string().min(1),
  display_name: z.string(),
  avatar_data_url: z.string().nullable(),
  given_name: z.string().nullable(),
  family_name: z.string().nullable(),
  nickname: z.string().nullable(),
  notes: z.string().nullable(),
  phones: z.array(ContactFieldValueSchema),
  emails: z.array(ContactFieldValueSchema),
  websites: z.array(ContactFieldValueSchema),
  birthday: z.string().nullable(),
  organization: z.string().nullable(),
  title: z.string().nullable(),
  addresses: z.array(ContactAddressSchema),
  source_id: z.string(),
  source_kind: z.string(),
  remote_id: z.string().nullable(),
  link_group_id: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string()
});

export const ContactDtoArraySchema = z.array(ContactDtoSchema);

export const ContactsListResultSchema = z.object({
  contacts: ContactDtoArraySchema,
  next_cursor: z.string().nullable(),
  total: z.number().int().min(0)
});

export const ContactCreateSchema = z.object({
  display_name: z.string().nullable(),
  avatar_data_url: z.string().nullable(),
  given_name: z.string().nullable(),
  family_name: z.string().nullable(),
  nickname: z.string().nullable(),
  notes: z.string().nullable(),
  phones: z.array(ContactFieldValueSchema),
  emails: z.array(ContactFieldValueSchema),
  websites: z.array(ContactFieldValueSchema),
  birthday: z.string().nullable(),
  organization: z.string().nullable(),
  title: z.string().nullable(),
  addresses: z.array(ContactAddressSchema),
  source_id: z.string().nullable(),
  source_kind: z.string().nullable(),
  remote_id: z.string().nullable(),
  link_group_id: z.string().nullable()
});

export const ContactPatchSchema = z.object({
  display_name: z.string().nullable().optional(),
  avatar_data_url: z.string().nullable().optional(),
  given_name: z.string().nullable().optional(),
  family_name: z.string().nullable().optional(),
  nickname: z.string().nullable().optional(),
  notes: z.string().nullable().optional(),
  phones: z.array(ContactFieldValueSchema).optional(),
  emails: z.array(ContactFieldValueSchema).optional(),
  websites: z.array(ContactFieldValueSchema).optional(),
  birthday: z.string().nullable().optional(),
  organization: z.string().nullable().optional(),
  title: z.string().nullable().optional(),
  addresses: z.array(ContactAddressSchema).optional(),
  source_id: z.string().nullable().optional(),
  source_kind: z.string().nullable().optional(),
  remote_id: z.string().nullable().optional(),
  link_group_id: z.string().nullable().optional()
});

export const ContactUpdateArgsSchema = z.object({
  id: z.string().min(1),
  patch: ContactPatchSchema
});

export const ContactDedupeCandidateGroupSchema = z.object({
  group_id: z.string(),
  reason: z.string(),
  score: z.number().int().min(0),
  contacts: ContactDtoArraySchema
});

export const ContactsDedupePreviewResultSchema = z.object({
  groups: z.array(ContactDedupeCandidateGroupSchema)
});

export const ContactOpenActionResultSchema = z.object({
  launched: z.boolean(),
  url: z.string().min(1)
});

export const ContactImportConflictSchema = z.object({
  imported: ContactDtoSchema,
  existing: ContactDtoSchema,
  score: z.number().int().min(0),
  reason: z.string()
});

export const ContactsImportPreviewResultSchema = z.object({
  batch_id: z.string(),
  source: z.string(),
  total_rows: z.number().int().min(0),
  valid_rows: z.number().int().min(0),
  skipped_rows: z.number().int().min(0),
  potential_duplicates: z.number().int().min(0),
  contacts: ContactDtoArraySchema,
  conflicts: z.array(ContactImportConflictSchema),
  errors: z.array(z.string())
});

export const ContactsImportCommitResultSchema = z.object({
  batch_id: z.string(),
  created: z.number().int().min(0),
  updated: z.number().int().min(0),
  skipped: z.number().int().min(0),
  failed: z.number().int().min(0),
  conflicts: z.number().int().min(0),
  errors: z.array(z.string())
});

export const ContactsMergeResultSchema = z.object({
  merged: ContactDtoSchema,
  removed_ids: z.array(z.string().min(1)),
  undo_id: z.string().min(1)
});

export const ContactsMergeUndoResultSchema = z.object({
  restored: z.number().int().min(0),
  undo_id: z.string().min(1)
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

export const ExternalCalendarCacheEntrySchema = z.object({
  cache_id: z.string().min(1),
  name: z.string(),
  location: z.string(),
  color: z.string(),
  cached_at: z.string(),
  kind: z.string()
});

export const ExternalCalendarCacheEntryArraySchema = z.array(ExternalCalendarCacheEntrySchema);

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
  notifications: z.object({
    due: z.object({
      enabled: z.boolean().optional(),
      pre_notify_enabled: z.boolean().optional(),
      pre_notify_minutes: z.number().int().optional(),
      scan_interval_seconds: z.number().int().optional()
    }).passthrough().optional()
  }).passthrough().optional(),
  ui: z.object({
    default_theme: z.string().optional(),
    theme: z.object({
      mode: z.string().optional(),
      follow_system: z.boolean().optional()
    }).passthrough().optional()
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
