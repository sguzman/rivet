import { invoke } from "@tauri-apps/api/core";
import type { ZodType } from "zod";

import { logger, setLoggerBridge } from "../lib/logger";
import {
  ContactCreateSchema,
  ContactDtoArraySchema,
  ContactDtoSchema,
  ContactOpenActionResultSchema,
  ContactUpdateArgsSchema,
  ContactsDedupePreviewResultSchema,
  ContactsImportCommitResultSchema,
  ContactsImportPreviewResultSchema,
  ContactsListResultSchema,
  ContactsMergeResultSchema,
  ContactsMergeUndoResultSchema,
  ExternalCalendarCacheEntryArraySchema,
  ExternalCalendarSourceSchema,
  ExternalCalendarSyncResultSchema,
  RivetRuntimeConfigSchema,
  TagSchemaSchema,
  TaskCreateSchema,
  TaskDtoArraySchema,
  TaskDtoSchema,
  TaskUpdateArgsSchema,
  describeSchemaError
} from "./schemas";
import type {
  ContactCreate,
  ContactDto,
  ContactIdArg,
  ContactOpenActionArgs,
  ContactOpenActionResult,
  ContactUpdateArgs,
  ContactsDedupePreviewArgs,
  ContactsDedupePreviewResult,
  ContactsDeleteBulkArgs,
  ContactsImportCommitArgs,
  ContactsImportCommitResult,
  ContactsImportPreviewArgs,
  ContactsImportPreviewResult,
  ContactsListArgs,
  ContactsListResult,
  ContactsMergeArgs,
  ContactsMergeResult,
  ContactsMergeUndoArgs,
  ContactsMergeUndoResult,
  ExternalCalendarCacheEntry,
  ExternalCalendarSource,
  ExternalCalendarSyncResult,
  TaskCreate,
  TaskDto,
  TaskIdArg,
  TasksListArgs,
  TaskUpdateArgs
} from "../types/core";
import type { RivetRuntimeConfig, TagSchema } from "../types/config";

const MOCK_TASKS_KEY = "rivet.mock.tasks";
const MOCK_CONTACTS_KEY = "rivet.mock.contacts";
const DEFAULT_TIMEOUT_MS = 30_000;
const EXTERNAL_CALENDAR_TIMEOUT_MS = 90_000;
const DEFAULT_TASK_QUERY: TasksListArgs = {
  query: null,
  status: null,
  project: null,
  tag: null
};
const DEFAULT_CONTACTS_QUERY: ContactsListArgs = {
  query: null,
  limit: 200,
  cursor: null,
  source: null,
  updated_after: null
};

export interface CommandFailureRecord {
  command: string;
  request_id: string;
  duration_ms: number;
  error: string;
  timestamp: string;
}

type CommandFailureSink = (record: CommandFailureRecord) => void;

let commandFailureSink: CommandFailureSink | null = null;

type RuntimeTransportMode = "auto" | "tauri" | "mock";
export interface ConfigEntryUpdate {
  section: string;
  key: string;
  value: string | number | boolean;
}

function resolveRuntimeTransportMode(): RuntimeTransportMode {
  const raw = String(import.meta.env.VITE_RIVET_UI_RUNTIME_MODE ?? "auto").trim().toLowerCase();
  if (raw === "tauri") {
    return "tauri";
  }
  if (raw === "mock") {
    return "mock";
  }
  return "auto";
}

const runtimeTransportMode = resolveRuntimeTransportMode();

export function setCommandFailureSink(sink: CommandFailureSink | null): void {
  commandFailureSink = sink;
}

const isTauriRuntime = (): boolean => {
  if (runtimeTransportMode === "mock") {
    return false;
  }
  if (runtimeTransportMode === "tauri") {
    return true;
  }
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
};

function parseWithSchema<T>(
  context: string,
  value: unknown,
  parser: ZodType<T>
): T {
  const result = parser.safeParse(value);
  if (!result.success) {
    const message = describeSchemaError(context, result.error);
    throw new Error(message);
  }
  return result.data;
}

function readLocalStorageJson(key: string): unknown {
  if (typeof window === "undefined") {
    return [];
  }
  const raw = window.localStorage.getItem(key);
  if (!raw) {
    return [];
  }
  try {
    return JSON.parse(raw);
  } catch {
    return [];
  }
}

function parseStoredTasks(): TaskDto[] {
  return parseWithSchema("mock.tasks", readLocalStorageJson(MOCK_TASKS_KEY), TaskDtoArraySchema);
}

function parseStoredContacts(): ContactDto[] {
  return parseWithSchema("mock.contacts", readLocalStorageJson(MOCK_CONTACTS_KEY), ContactDtoArraySchema);
}

function writeStorageJson(key: string, value: unknown): void {
  if (typeof window === "undefined") {
    return;
  }
  window.localStorage.setItem(key, JSON.stringify(value));
}

function writeStoredTasks(tasks: TaskDto[]): void {
  writeStorageJson(MOCK_TASKS_KEY, tasks);
}

function writeStoredContacts(contacts: ContactDto[]): void {
  writeStorageJson(MOCK_CONTACTS_KEY, contacts);
}

function makeMockTask(input: TaskCreate): TaskDto {
  const now = new Date().toISOString();
  return {
    uuid: crypto.randomUUID(),
    id: null,
    title: input.title,
    description: input.description,
    status: "Pending",
    project: input.project,
    tags: input.tags,
    priority: input.priority,
    due: input.due,
    wait: input.wait,
    scheduled: input.scheduled,
    created: now,
    modified: now
  };
}

function makeMockContact(input: ContactCreate): ContactDto {
  const now = new Date().toISOString();
  const firstEmail = input.emails.find((item) => item.value.trim().length > 0)?.value ?? "";
  const firstPhone = input.phones.find((item) => item.value.trim().length > 0)?.value ?? "";
  const display = input.display_name?.trim() || [input.given_name ?? "", input.family_name ?? ""].join(" ").trim() || firstEmail || firstPhone || "Unnamed Contact";

  return {
    id: crypto.randomUUID(),
    display_name: display,
    avatar_data_url: input.avatar_data_url ?? null,
    given_name: input.given_name,
    family_name: input.family_name,
    nickname: input.nickname,
    notes: input.notes,
    phones: input.phones,
    emails: input.emails,
    websites: input.websites,
    birthday: input.birthday,
    organization: input.organization,
    title: input.title,
    addresses: input.addresses,
    source_id: input.source_id ?? "local",
    source_kind: input.source_kind ?? "local",
    remote_id: input.remote_id,
    link_group_id: input.link_group_id,
    created_at: now,
    updated_at: now
  };
}

function contactSearchMatches(contact: ContactDto, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) {
    return true;
  }

  const fields = [
    contact.display_name,
    contact.given_name ?? "",
    contact.family_name ?? "",
    contact.nickname ?? "",
    contact.notes ?? "",
    contact.organization ?? "",
    ...contact.emails.map((item) => item.value),
    ...contact.phones.map((item) => item.value)
  ].join(" ").toLowerCase();

  return fields.includes(q);
}

async function invokeCommand<R>(command: string, args?: unknown): Promise<R> {
  const requestId = crypto.randomUUID();
  const startedAt = performance.now();
  const timeoutMs = command.startsWith("external_calendar_") ? EXTERNAL_CALENDAR_TIMEOUT_MS : DEFAULT_TIMEOUT_MS;
  const instrumentCommand = command !== "ui_log";
  if (instrumentCommand) {
    logger.debug("invoke.start", `${command} request_id=${requestId}`);
  }

  const run = async (): Promise<R> => {
    if (isTauriRuntime()) {
      const payload = typeof args === "undefined" ? undefined : args;
      if (typeof payload === "undefined") {
        return invoke<R>(command, { request_id: requestId });
      }
      return invoke<R>(command, { args: payload, request_id: requestId });
    }

    switch (command) {
      case "tasks_list": {
        return parseStoredTasks() as R;
      }
      case "task_add": {
        const payload = args as TaskCreate;
        const tasks = parseStoredTasks();
        const task = makeMockTask(payload);
        tasks.unshift(task);
        writeStoredTasks(tasks);
        return task as R;
      }
      case "task_done": {
        const payload = args as TaskIdArg;
        const tasks = parseStoredTasks().map((entry) => {
          if (entry.uuid !== payload.uuid) {
            return entry;
          }
          return {
            ...entry,
            status: "Completed" as const,
            modified: new Date().toISOString()
          };
        });
        writeStoredTasks(tasks);
        const target = tasks.find((entry) => entry.uuid === payload.uuid);
        if (!target) {
          throw new Error(`task not found: ${payload.uuid}`);
        }
        return target as R;
      }
      case "task_uncomplete": {
        const payload = args as TaskIdArg;
        const tasks = parseStoredTasks().map((entry) => {
          if (entry.uuid !== payload.uuid) {
            return entry;
          }
          return {
            ...entry,
            status: "Pending" as const,
            modified: new Date().toISOString()
          };
        });
        writeStoredTasks(tasks);
        const target = tasks.find((entry) => entry.uuid === payload.uuid);
        if (!target) {
          throw new Error(`task not found: ${payload.uuid}`);
        }
        return target as R;
      }
      case "task_delete": {
        const payload = args as TaskIdArg;
        const tasks = parseStoredTasks().filter((entry) => entry.uuid !== payload.uuid);
        writeStoredTasks(tasks);
        return undefined as R;
      }
      case "task_update": {
        const payload = args as TaskUpdateArgs;
        const tasks = parseStoredTasks().map((entry) => {
          if (entry.uuid !== payload.uuid) {
            return entry;
          }

          return {
            ...entry,
            title: payload.patch.title ?? entry.title,
            description: payload.patch.description ?? entry.description,
            project: typeof payload.patch.project === "undefined" ? entry.project : payload.patch.project,
            tags: payload.patch.tags ?? entry.tags,
            priority: typeof payload.patch.priority === "undefined" ? entry.priority : payload.patch.priority,
            due: typeof payload.patch.due === "undefined" ? entry.due : payload.patch.due,
            wait: typeof payload.patch.wait === "undefined" ? entry.wait : payload.patch.wait,
            scheduled: typeof payload.patch.scheduled === "undefined" ? entry.scheduled : payload.patch.scheduled,
            modified: new Date().toISOString()
          };
        });
        writeStoredTasks(tasks);
        const target = tasks.find((entry) => entry.uuid === payload.uuid);
        if (!target) {
          throw new Error(`task not found: ${payload.uuid}`);
        }
        return target as R;
      }
      case "contacts_list": {
        const payload = (args ?? DEFAULT_CONTACTS_QUERY) as ContactsListArgs;
        const all = parseStoredContacts();
        const filtered = all.filter((contact) => contactSearchMatches(contact, payload.query ?? ""));
        const total = filtered.length;
        const limit = Math.max(1, payload.limit ?? 200);
        const offset = Number(payload.cursor ?? "0") || 0;
        const contacts = filtered.slice(offset, offset + limit);
        const next_cursor = offset + contacts.length < total ? String(offset + contacts.length) : null;
        return { contacts, next_cursor, total } as R;
      }
      case "contact_add": {
        const payload = args as ContactCreate;
        const contacts = parseStoredContacts();
        const created = makeMockContact(payload);
        contacts.unshift(created);
        writeStoredContacts(contacts);
        return created as R;
      }
      case "contact_update": {
        const payload = args as ContactUpdateArgs;
        const contacts = parseStoredContacts().map((entry) => {
          if (entry.id !== payload.id) {
            return entry;
          }
          return {
            ...entry,
            display_name: typeof payload.patch.display_name === "undefined" ? entry.display_name : (payload.patch.display_name ?? ""),
            avatar_data_url: typeof payload.patch.avatar_data_url === "undefined" ? entry.avatar_data_url : payload.patch.avatar_data_url,
            given_name: typeof payload.patch.given_name === "undefined" ? entry.given_name : payload.patch.given_name,
            family_name: typeof payload.patch.family_name === "undefined" ? entry.family_name : payload.patch.family_name,
            nickname: typeof payload.patch.nickname === "undefined" ? entry.nickname : payload.patch.nickname,
            notes: typeof payload.patch.notes === "undefined" ? entry.notes : payload.patch.notes,
            phones: payload.patch.phones ?? entry.phones,
            emails: payload.patch.emails ?? entry.emails,
            websites: payload.patch.websites ?? entry.websites,
            birthday: typeof payload.patch.birthday === "undefined" ? entry.birthday : payload.patch.birthday,
            organization: typeof payload.patch.organization === "undefined" ? entry.organization : payload.patch.organization,
            title: typeof payload.patch.title === "undefined" ? entry.title : payload.patch.title,
            addresses: payload.patch.addresses ?? entry.addresses,
            source_id: typeof payload.patch.source_id === "undefined" ? entry.source_id : (payload.patch.source_id ?? ""),
            source_kind: typeof payload.patch.source_kind === "undefined" ? entry.source_kind : (payload.patch.source_kind ?? ""),
            remote_id: typeof payload.patch.remote_id === "undefined" ? entry.remote_id : payload.patch.remote_id,
            link_group_id: typeof payload.patch.link_group_id === "undefined" ? entry.link_group_id : payload.patch.link_group_id,
            updated_at: new Date().toISOString()
          };
        });
        writeStoredContacts(contacts);
        const target = contacts.find((entry) => entry.id === payload.id);
        if (!target) {
          throw new Error(`contact not found: ${payload.id}`);
        }
        return target as R;
      }
      case "contact_delete": {
        const payload = args as ContactIdArg;
        const contacts = parseStoredContacts().filter((entry) => entry.id !== payload.id);
        writeStoredContacts(contacts);
        return undefined as R;
      }
      case "contacts_delete_bulk": {
        const payload = args as ContactsDeleteBulkArgs;
        const ids = new Set(payload.ids);
        const contacts = parseStoredContacts();
        const kept = contacts.filter((entry) => !ids.has(entry.id));
        const deleted = contacts.length - kept.length;
        writeStoredContacts(kept);
        return deleted as R;
      }
      case "contacts_dedupe_preview":
      case "contacts_dedupe_candidates": {
        const all = parseStoredContacts();
        const groups = new Map<string, ContactDto[]>();
        for (const contact of all) {
          const key = contact.display_name.trim().toLowerCase();
          if (!key) {
            continue;
          }
          const current = groups.get(key) ?? [];
          current.push(contact);
          groups.set(key, current);
        }
        const out = [...groups.entries()]
          .filter(([, contacts]) => contacts.length > 1)
          .map(([key, contacts], index) => ({
            group_id: `mock-${index}`,
            reason: `same name: ${key}`,
            score: 60,
            contacts
          }));
        return { groups: out } as R;
      }
      case "contact_open_action": {
        const payload = args as ContactOpenActionArgs;
        const url = payload.action === "tel" || payload.action === "phone"
          ? `tel:${payload.value ?? ""}`
          : `mailto:${payload.value ?? ""}`;
        return {
          launched: false,
          url
        } as R;
      }
      case "contacts_import_preview": {
        const payload = args as ContactsImportPreviewArgs;
        return {
          batch_id: crypto.randomUUID(),
          source: payload.source,
          total_rows: 0,
          valid_rows: 0,
          skipped_rows: 0,
          potential_duplicates: 0,
          contacts: [],
          conflicts: [],
          errors: []
        } as R;
      }
      case "contacts_import_commit": {
        return {
          batch_id: crypto.randomUUID(),
          created: 0,
          updated: 0,
          skipped: 0,
          failed: 0,
          conflicts: 0,
          errors: []
        } as R;
      }
      case "contacts_merge": {
        const payload = args as ContactsMergeArgs;
        const contacts = parseStoredContacts();
        const ids = new Set(payload.ids);
        const selected = contacts.filter((entry) => ids.has(entry.id));
        if (selected.length < 2) {
          throw new Error("need at least two contacts to merge");
        }
        const targetId = payload.target_id ?? selected[0].id;
        const target = selected.find((entry) => entry.id === targetId) ?? selected[0];
        const removed = selected.filter((entry) => entry.id !== target.id).map((entry) => entry.id);
        const next = contacts.filter((entry) => !removed.includes(entry.id));
        writeStoredContacts(next);
        return {
          merged: target,
          removed_ids: removed,
          undo_id: crypto.randomUUID()
        } as R;
      }
      case "contacts_merge_undo": {
        return {
          restored: 0,
          undo_id: (args as ContactsMergeUndoArgs)?.undo_id ?? ""
        } as R;
      }
      case "config_snapshot": {
        return {} as R;
      }
      case "config_apply_updates": {
        return {} as R;
      }
      case "external_calendar_cache_list": {
        return [] as R;
      }
      case "external_calendar_import_cached": {
        const payload = args as { source: ExternalCalendarSource; cache_id: string };
        return {
          calendar_id: payload.source.id,
          created: 0,
          updated: 0,
          deleted: 0,
          remote_events: 0,
          refresh_minutes: payload.source.refresh_minutes
        } as R;
      }
      case "tag_schema_snapshot": {
        return { version: 1, keys: [] } as R;
      }
      case "ui_log": {
        return undefined as R;
      }
      default:
        throw new Error(`unsupported mock command: ${command}`);
    }
  };

  let timeoutId: number | null = null;
  const timeout = new Promise<never>((_, reject) => {
    timeoutId = window.setTimeout(() => {
      reject(new Error(`invoke timeout (${command}) after ${timeoutMs}ms request_id=${requestId}`));
    }, timeoutMs);
  });

  try {
    const result = await Promise.race([run(), timeout]);
    const elapsed = Math.round((performance.now() - startedAt) * 100) / 100;
    if (instrumentCommand) {
      logger.info("invoke.success", `${command} request_id=${requestId} duration_ms=${elapsed}`);
    }
    return result;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const elapsed = Math.round((performance.now() - startedAt) * 100) / 100;
    if (instrumentCommand) {
      logger.error("invoke.error", `${command} request_id=${requestId} duration_ms=${elapsed} error=${message}`);
      commandFailureSink?.({
        command,
        request_id: requestId,
        duration_ms: elapsed,
        error: message,
        timestamp: new Date().toISOString()
      });
    }
    throw error;
  } finally {
    if (timeoutId !== null) {
      window.clearTimeout(timeoutId);
    }
  }
}

setLoggerBridge(async (event, detail) => {
  try {
    await invokeCommand<void>("ui_log", { event, detail });
  } catch {
    // avoid recursive logger calls for logging failures.
  }
});

export async function healthCheck(): Promise<void> {
  const response = await invokeCommand<unknown>("tasks_list", DEFAULT_TASK_QUERY);
  parseWithSchema("tasks_list healthcheck", response, TaskDtoArraySchema);
}

export async function listTasks(args: TasksListArgs = DEFAULT_TASK_QUERY): Promise<TaskDto[]> {
  const response = await invokeCommand<unknown>("tasks_list", args);
  return parseWithSchema("tasks_list response", response, TaskDtoArraySchema);
}

export async function addTask(args: TaskCreate): Promise<TaskDto> {
  logger.info("invoke.task_add", "adding task from React shell");
  const payload = parseWithSchema("task_add args", args, TaskCreateSchema);
  const response = await invokeCommand<unknown>("task_add", payload);
  return parseWithSchema("task_add response", response, TaskDtoSchema);
}

export async function updateTask(args: TaskUpdateArgs): Promise<TaskDto> {
  const payload = parseWithSchema("task_update args", args, TaskUpdateArgsSchema);
  const response = await invokeCommand<unknown>("task_update", payload);
  return parseWithSchema("task_update response", response, TaskDtoSchema);
}

export async function doneTask(uuid: string): Promise<TaskDto> {
  const response = await invokeCommand<unknown>("task_done", { uuid });
  return parseWithSchema("task_done response", response, TaskDtoSchema);
}

export async function uncompleteTask(uuid: string): Promise<TaskDto> {
  const response = await invokeCommand<unknown>("task_uncomplete", { uuid });
  return parseWithSchema("task_uncomplete response", response, TaskDtoSchema);
}

export async function deleteTask(uuid: string): Promise<void> {
  return invokeCommand<void>("task_delete", { uuid });
}

export async function listContacts(args: ContactsListArgs = DEFAULT_CONTACTS_QUERY): Promise<ContactsListResult> {
  const response = await invokeCommand<unknown>("contacts_list", args);
  return parseWithSchema("contacts_list response", response, ContactsListResultSchema);
}

export async function addContact(args: ContactCreate): Promise<ContactDto> {
  const payload = parseWithSchema("contact_add args", args, ContactCreateSchema);
  const response = await invokeCommand<unknown>("contact_add", payload);
  return parseWithSchema("contact_add response", response, ContactDtoSchema);
}

export async function updateContact(args: ContactUpdateArgs): Promise<ContactDto> {
  const payload = parseWithSchema("contact_update args", args, ContactUpdateArgsSchema);
  const response = await invokeCommand<unknown>("contact_update", payload);
  return parseWithSchema("contact_update response", response, ContactDtoSchema);
}

export async function deleteContact(id: string): Promise<void> {
  return invokeCommand<void>("contact_delete", { id });
}

export async function deleteContactsBulk(args: ContactsDeleteBulkArgs): Promise<number> {
  const response = await invokeCommand<unknown>("contacts_delete_bulk", args);
  return Number(response);
}

export async function previewContactsDedupe(args: ContactsDedupePreviewArgs): Promise<ContactsDedupePreviewResult> {
  const response = await invokeCommand<unknown>("contacts_dedupe_preview", args);
  return parseWithSchema("contacts_dedupe_preview response", response, ContactsDedupePreviewResultSchema);
}

export async function listContactsDedupeCandidates(args: ContactsDedupePreviewArgs): Promise<ContactsDedupePreviewResult> {
  const response = await invokeCommand<unknown>("contacts_dedupe_candidates", args);
  return parseWithSchema("contacts_dedupe_candidates response", response, ContactsDedupePreviewResultSchema);
}

export async function openContactAction(args: ContactOpenActionArgs): Promise<ContactOpenActionResult> {
  const response = await invokeCommand<unknown>("contact_open_action", args);
  return parseWithSchema("contact_open_action response", response, ContactOpenActionResultSchema);
}

export async function previewContactsImport(args: ContactsImportPreviewArgs): Promise<ContactsImportPreviewResult> {
  const response = await invokeCommand<unknown>("contacts_import_preview", args);
  return parseWithSchema("contacts_import_preview response", response, ContactsImportPreviewResultSchema);
}

export async function commitContactsImport(args: ContactsImportCommitArgs): Promise<ContactsImportCommitResult> {
  const response = await invokeCommand<unknown>("contacts_import_commit", args);
  return parseWithSchema("contacts_import_commit response", response, ContactsImportCommitResultSchema);
}

export async function mergeContacts(args: ContactsMergeArgs): Promise<ContactsMergeResult> {
  const response = await invokeCommand<unknown>("contacts_merge", args);
  return parseWithSchema("contacts_merge response", response, ContactsMergeResultSchema);
}

export async function undoContactsMerge(args: ContactsMergeUndoArgs): Promise<ContactsMergeUndoResult> {
  const response = await invokeCommand<unknown>("contacts_merge_undo", args);
  return parseWithSchema("contacts_merge_undo response", response, ContactsMergeUndoResultSchema);
}

export async function syncExternalCalendar(source: ExternalCalendarSource): Promise<ExternalCalendarSyncResult> {
  const payload = parseWithSchema("external_calendar_sync args", source, ExternalCalendarSourceSchema);
  const response = await invokeCommand<unknown>("external_calendar_sync", payload);
  return parseWithSchema("external_calendar_sync response", response, ExternalCalendarSyncResultSchema);
}

export async function importExternalCalendarIcs(source: ExternalCalendarSource, icsText: string): Promise<ExternalCalendarSyncResult> {
  const payload = {
    source: parseWithSchema("external_calendar_import_ics args source", source, ExternalCalendarSourceSchema),
    ics_text: icsText
  };
  const response = await invokeCommand<unknown>("external_calendar_import_ics", payload);
  return parseWithSchema("external_calendar_import_ics response", response, ExternalCalendarSyncResultSchema);
}

export async function listExternalCalendarCache(): Promise<ExternalCalendarCacheEntry[]> {
  const response = await invokeCommand<unknown>("external_calendar_cache_list");
  return parseWithSchema("external_calendar_cache_list response", response, ExternalCalendarCacheEntryArraySchema);
}

export async function importExternalCalendarCached(source: ExternalCalendarSource, cacheId: string): Promise<ExternalCalendarSyncResult> {
  const payload = {
    source: parseWithSchema("external_calendar_import_cached args source", source, ExternalCalendarSourceSchema),
    cache_id: cacheId
  };
  const response = await invokeCommand<unknown>("external_calendar_import_cached", payload);
  return parseWithSchema("external_calendar_import_cached response", response, ExternalCalendarSyncResultSchema);
}

export async function loadConfigSnapshot(): Promise<RivetRuntimeConfig> {
  try {
    const response = await invokeCommand<unknown>("config_snapshot");
    return parseWithSchema("config_snapshot response", response, RivetRuntimeConfigSchema);
  } catch (error) {
    logger.warn("config_snapshot", String(error));
    return {};
  }
}

export async function applyConfigUpdates(updates: ConfigEntryUpdate[]): Promise<RivetRuntimeConfig> {
  const payload = {
    updates
  };
  const response = await invokeCommand<unknown>("config_apply_updates", payload);
  return parseWithSchema("config_apply_updates response", response, RivetRuntimeConfigSchema);
}

export async function loadTagSchemaSnapshot(): Promise<TagSchema> {
  try {
    const response = await invokeCommand<unknown>("tag_schema_snapshot");
    return parseWithSchema("tag_schema_snapshot response", response, TagSchemaSchema);
  } catch (error) {
    logger.warn("tag_schema_snapshot", String(error));
    return { version: 1, keys: [] };
  }
}
