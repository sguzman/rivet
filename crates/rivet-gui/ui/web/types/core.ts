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

export interface ContactFieldValue {
  value: string;
  kind: string;
  is_primary: boolean;
}

export interface ContactAddress {
  kind: string;
  street: string;
  city: string;
  region: string;
  postal_code: string;
  country: string;
}

export interface ContactDto {
  id: string;
  display_name: string;
  avatar_data_url: string | null;
  import_batch_id: string | null;
  source_file_name: string | null;
  given_name: string | null;
  family_name: string | null;
  nickname: string | null;
  notes: string | null;
  phones: ContactFieldValue[];
  emails: ContactFieldValue[];
  websites: ContactFieldValue[];
  birthday: string | null;
  organization: string | null;
  title: string | null;
  addresses: ContactAddress[];
  source_id: string;
  source_kind: string;
  remote_id: string | null;
  link_group_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface ContactsListArgs {
  query: string | null;
  limit: number | null;
  cursor: string | null;
  source: string | null;
  updated_after: string | null;
}

export interface ContactsListResult {
  contacts: ContactDto[];
  next_cursor: string | null;
  total: number;
}

export interface ContactCreate {
  display_name: string | null;
  avatar_data_url: string | null;
  import_batch_id: string | null;
  source_file_name: string | null;
  given_name: string | null;
  family_name: string | null;
  nickname: string | null;
  notes: string | null;
  phones: ContactFieldValue[];
  emails: ContactFieldValue[];
  websites: ContactFieldValue[];
  birthday: string | null;
  organization: string | null;
  title: string | null;
  addresses: ContactAddress[];
  source_id: string | null;
  source_kind: string | null;
  remote_id: string | null;
  link_group_id: string | null;
}

export interface ContactPatch {
  display_name?: string | null;
  avatar_data_url?: string | null;
  import_batch_id?: string | null;
  source_file_name?: string | null;
  given_name?: string | null;
  family_name?: string | null;
  nickname?: string | null;
  notes?: string | null;
  phones?: ContactFieldValue[];
  emails?: ContactFieldValue[];
  websites?: ContactFieldValue[];
  birthday?: string | null;
  organization?: string | null;
  title?: string | null;
  addresses?: ContactAddress[];
  source_id?: string | null;
  source_kind?: string | null;
  remote_id?: string | null;
  link_group_id?: string | null;
}

export interface ContactUpdateArgs {
  id: string;
  patch: ContactPatch;
}

export interface ContactIdArg {
  id: string;
}

export interface ContactsDeleteBulkArgs {
  ids: string[];
}

export interface ContactsDedupePreviewArgs {
  query: string | null;
}

export interface ContactDedupeCandidateGroup {
  group_id: string;
  reason: string;
  score: number;
  contacts: ContactDto[];
}

export interface ContactsDedupePreviewResult {
  groups: ContactDedupeCandidateGroup[];
}

export interface ContactsDedupeDecideArgs {
  candidate_group_id: string;
  decision: "ignored" | "separate" | "merged" | string;
  actor: string | null;
}

export interface ContactsDedupeDecideResult {
  candidate_group_id: string;
  decision: string;
  actor: string;
  decided_at: string;
}

export interface ContactOpenActionArgs {
  id: string;
  action: "mailto" | "email" | "tel" | "phone";
  value: string | null;
}

export interface ContactOpenActionResult {
  launched: boolean;
  url: string;
}

export interface ContactImportConflict {
  imported: ContactDto;
  existing: ContactDto;
  score: number;
  reason: string;
}

export interface ContactsImportPreviewArgs {
  source: string;
  file_name: string | null;
  content: string;
}

export interface ContactsImportPreviewResult {
  batch_id: string;
  source: string;
  total_rows: number;
  valid_rows: number;
  skipped_rows: number;
  potential_duplicates: number;
  contacts: ContactDto[];
  conflicts: ContactImportConflict[];
  errors: string[];
}

export interface ContactsImportCommitArgs {
  source: string;
  file_name: string | null;
  content: string;
  mode: "safe" | "upsert" | "review";
}

export interface ContactsImportCommitResult {
  batch_id: string;
  created: number;
  updated: number;
  skipped: number;
  failed: number;
  conflicts: number;
  errors: string[];
}

export interface ContactsMergeArgs {
  ids: string[];
  target_id: string | null;
}

export interface ContactsMergeResult {
  merged: ContactDto;
  removed_ids: string[];
  undo_id: string;
}

export interface ContactsMergeUndoArgs {
  undo_id: string | null;
}

export interface ContactsMergeUndoResult {
  restored: number;
  undo_id: string;
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
