import { invoke } from "@tauri-apps/api/core";
import type { ZodType } from "zod";

import { logger, setLoggerBridge } from "../lib/logger";
import {
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
const DEFAULT_TIMEOUT_MS = 30_000;
const EXTERNAL_CALENDAR_TIMEOUT_MS = 90_000;
const DEFAULT_TASK_QUERY: TasksListArgs = {
  query: null,
  status: null,
  project: null,
  tag: null
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

function parseStoredTasks(): TaskDto[] {
  if (typeof window === "undefined") {
    return [];
  }

  const raw = window.localStorage.getItem(MOCK_TASKS_KEY);
  if (!raw) {
    return [];
  }

  try {
    const parsed = JSON.parse(raw);
    return parseWithSchema("mock.tasks", parsed, TaskDtoArraySchema);
  } catch {
    return [];
  }
}

function writeStoredTasks(tasks: TaskDto[]): void {
  if (typeof window === "undefined") {
    return;
  }
  window.localStorage.setItem(MOCK_TASKS_KEY, JSON.stringify(tasks));
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
      case "config_snapshot": {
        return {} as R;
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

export async function deleteTask(uuid: string): Promise<void> {
  return invokeCommand<void>("task_delete", { uuid });
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

export async function loadConfigSnapshot(): Promise<RivetRuntimeConfig> {
  try {
    const response = await invokeCommand<unknown>("config_snapshot");
    return parseWithSchema("config_snapshot response", response, RivetRuntimeConfigSchema);
  } catch (error) {
    logger.warn("config_snapshot", String(error));
    return {};
  }
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
