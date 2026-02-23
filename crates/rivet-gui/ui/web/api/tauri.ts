import { invoke } from "@tauri-apps/api/core";

import { logger, setLoggerBridge } from "../lib/logger";
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
const DEFAULT_TASK_QUERY: TasksListArgs = {
  query: null,
  status: null,
  project: null,
  tag: null
};

const isTauriRuntime = (): boolean => {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
};

function parseStoredTasks(): TaskDto[] {
  if (typeof window === "undefined") {
    return [];
  }

  const raw = window.localStorage.getItem(MOCK_TASKS_KEY);
  if (!raw) {
    return [];
  }

  try {
    const parsed = JSON.parse(raw) as TaskDto[];
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed;
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
  if (isTauriRuntime()) {
    if (typeof args === "undefined") {
      return invoke<R>(command);
    }
    return invoke<R>(command, { args });
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
}

setLoggerBridge(async (event, detail) => {
  try {
    await invokeCommand<void>("ui_log", { event, detail });
  } catch {
    // avoid recursive logger calls for logging failures.
  }
});

export async function healthCheck(): Promise<void> {
  await invokeCommand<TaskDto[]>("tasks_list", DEFAULT_TASK_QUERY);
}

export async function listTasks(args: TasksListArgs = DEFAULT_TASK_QUERY): Promise<TaskDto[]> {
  return invokeCommand<TaskDto[]>("tasks_list", args);
}

export async function addTask(args: TaskCreate): Promise<TaskDto> {
  logger.info("invoke.task_add", "adding task from React shell");
  return invokeCommand<TaskDto>("task_add", args);
}

export async function updateTask(args: TaskUpdateArgs): Promise<TaskDto> {
  return invokeCommand<TaskDto>("task_update", args);
}

export async function doneTask(uuid: string): Promise<TaskDto> {
  return invokeCommand<TaskDto>("task_done", { uuid });
}

export async function deleteTask(uuid: string): Promise<void> {
  return invokeCommand<void>("task_delete", { uuid });
}

export async function syncExternalCalendar(source: ExternalCalendarSource): Promise<ExternalCalendarSyncResult> {
  return invokeCommand<ExternalCalendarSyncResult>("external_calendar_sync", source);
}

export async function importExternalCalendarIcs(source: ExternalCalendarSource, icsText: string): Promise<ExternalCalendarSyncResult> {
  return invokeCommand<ExternalCalendarSyncResult>("external_calendar_import_ics", {
    source,
    ics_text: icsText
  });
}

export async function loadConfigSnapshot(): Promise<RivetRuntimeConfig> {
  try {
    return await invokeCommand<RivetRuntimeConfig>("config_snapshot");
  } catch (error) {
    logger.warn("config_snapshot", String(error));
    return {};
  }
}

export async function loadTagSchemaSnapshot(): Promise<TagSchema> {
  try {
    return await invokeCommand<TagSchema>("tag_schema_snapshot");
  } catch (error) {
    logger.warn("tag_schema_snapshot", String(error));
    return { version: 1, keys: [] };
  }
}
