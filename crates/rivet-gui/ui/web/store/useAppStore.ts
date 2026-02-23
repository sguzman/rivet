import { useMemo } from "react";
import { create } from "zustand";

import {
  addTask,
  type CommandFailureRecord,
  deleteTask,
  doneTask,
  healthCheck,
  importExternalCalendarIcs,
  listTasks,
  loadConfigSnapshot,
  loadTagSchemaSnapshot,
  setCommandFailureSink,
  syncExternalCalendar,
  uncompleteTask,
  updateTask
} from "../api/tauri";
import {
  canManuallyCompleteTask,
  calendarDateFromIso,
  calendarDateToIso,
  collectCalendarDueTasks,
  isCalendarEventTask,
  resolveCalendarConfig,
  shiftCalendarFocus as shiftFocusDate,
  todayInTimezone
} from "../lib/calendar";
import { logger } from "../lib/logger";
import {
  browserDueNotificationPermission,
  collectDueNotificationEvents,
  defaultDueNotificationConfig,
  emitDueNotification,
  requestDueNotificationPermission as requestBrowserDueNotificationPermission,
  sanitizeDueNotificationConfig,
  type DueNotificationPermission
} from "../lib/notifications";
import {
  CALENDAR_VIEW_STORAGE_KEY,
  DUE_NOTIFICATION_SENT_STORAGE_KEY,
  EXTERNAL_CALENDARS_STORAGE_KEY,
  KANBAN_ACTIVE_BOARD_STORAGE_KEY,
  KANBAN_BOARDS_STORAGE_KEY,
  THEME_STORAGE_KEY,
  WORKSPACE_TAB_STORAGE_KEY,
  assignUniqueExternalCalendarColors,
  loadNotificationSentRegistry,
  loadNotificationSettings,
  loadActiveKanbanBoardId,
  loadExternalCalendars,
  loadKanbanBoards,
  loadKanbanCompactCards,
  makeUniqueBoardName,
  newExternalCalendarSource,
  nextBoardColor,
  saveActiveKanbanBoardId,
  saveExternalCalendars,
  saveKanbanBoards,
  saveKanbanCompactCards,
  saveNotificationSentRegistry,
  saveNotificationSettings
} from "../lib/storage";
import {
  BOARD_TAG_KEY,
  boardIdFromTaskTags,
  buildTagColorMap,
  collectTagsForSubmit,
  defaultKanbanLane,
  kanbanColumnsFromSchema,
  pushTagUnique,
  removeTagsForKey,
  splitTags,
  tagsForKanbanMove,
  taskHasTagValue
} from "../lib/tags";
import { buildTaskFacets, filterTasks } from "./selectors";
import type { RivetRuntimeConfig, TagSchema } from "../types/config";
import type { ExternalCalendarSource, TaskCreate, TaskDto, TaskPatch } from "../types/core";
import type { AddTaskDialogContext, DueFilter, DueNotificationConfig, PriorityFilter, RecurrenceDraft, StatusFilter, TaskFilters, ThemeMode, WorkspaceTab } from "../types/ui";

function readStorageString(key: string): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function writeStorageString(key: string, value: string): void {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.setItem(key, value);
  } catch (error) {
    logger.warn("storage.write", `${key}: ${String(error)}`);
  }
}

function loadThemeMode(): ThemeMode {
  const raw = readStorageString(THEME_STORAGE_KEY);
  return raw === "night" ? "night" : "day";
}

function saveThemeMode(mode: ThemeMode): void {
  writeStorageString(THEME_STORAGE_KEY, mode);
}

function loadWorkspaceTab(): WorkspaceTab {
  const raw = readStorageString(WORKSPACE_TAB_STORAGE_KEY);
  if (raw === "kanban" || raw === "calendar") {
    return raw;
  }
  return "tasks";
}

function saveWorkspaceTab(tab: WorkspaceTab): void {
  writeStorageString(WORKSPACE_TAB_STORAGE_KEY, tab);
}

function loadCalendarViewMode(): "year" | "quarter" | "month" | "week" | "day" {
  const raw = readStorageString(CALENDAR_VIEW_STORAGE_KEY);
  if (raw === "year" || raw === "quarter" || raw === "month" || raw === "week" || raw === "day") {
    return raw;
  }
  return "month";
}

function saveCalendarViewMode(view: "year" | "quarter" | "month" | "week" | "day"): void {
  writeStorageString(CALENDAR_VIEW_STORAGE_KEY, view);
}

function loadExternalCalendarsSafe(): ExternalCalendarSource[] {
  const sources = loadExternalCalendars();
  logger.info("external_calendars.load", `loaded ${sources.length} local sources from ${EXTERNAL_CALENDARS_STORAGE_KEY}`);
  return sources;
}

function loadKanbanBoardsSafe() {
  const boards = loadKanbanBoards();
  logger.info("kanban_boards.load", `loaded ${boards.length} local boards from ${KANBAN_BOARDS_STORAGE_KEY}`);
  return boards;
}

function loadActiveBoardSafe(boards: Array<{ id: string; name: string; color: string }>): string | null {
  const active = loadActiveKanbanBoardId(boards);
  if (active) {
    logger.debug("kanban_boards.active", `${KANBAN_ACTIVE_BOARD_STORAGE_KEY}=${active}`);
  }
  return active;
}

function emptyTaskFilters(): TaskFilters {
  return {
    search: "",
    status: "all",
    project: "",
    tag: "",
    priority: "all",
    due: "all"
  };
}

interface AppState {
  bootstrapped: boolean;
  activeTab: WorkspaceTab;
  themeMode: ThemeMode;
  loading: boolean;
  error: string | null;
  tasks: TaskDto[];
  selectedTaskId: string | null;
  addTaskDialogOpen: boolean;
  addTaskDialogContext: AddTaskDialogContext;
  taskFilters: TaskFilters;
  kanbanFilters: TaskFilters;
  runtimeConfig: RivetRuntimeConfig | null;
  tagSchema: TagSchema | null;
  tagColorMap: Record<string, string>;
  kanbanBoards: Array<{ id: string; name: string; color: string }>;
  activeKanbanBoardId: string | null;
  kanbanCompactCards: boolean;
  draggingKanbanTaskId: string | null;
  dragOverKanbanLane: string | null;
  calendarView: "year" | "quarter" | "month" | "week" | "day";
  calendarFocusDateIso: string;
  calendarTaskFilter: string;
  externalCalendars: ExternalCalendarSource[];
  externalCalendarBusy: boolean;
  externalCalendarLastSync: string | null;
  settingsOpen: boolean;
  dueNotificationConfig: DueNotificationConfig;
  dueNotificationPermission: DueNotificationPermission;
  dueNotificationSent: string[];
  commandFailures: CommandFailureRecord[];

  bootstrap: () => Promise<void>;
  loadTasks: () => Promise<void>;

  setActiveTab: (tab: WorkspaceTab) => void;
  toggleTheme: () => void;
  selectTask: (taskId: string | null) => void;

  setTaskSearchFilter: (value: string) => void;
  setTaskStatusFilter: (value: StatusFilter) => void;
  setTaskProjectFilter: (value: string) => void;
  setTaskTagFilter: (value: string) => void;
  setTaskPriorityFilter: (value: PriorityFilter) => void;
  setTaskDueFilter: (value: DueFilter) => void;
  clearTaskFilters: () => void;

  setKanbanStatusFilter: (value: StatusFilter) => void;
  setKanbanProjectFilter: (value: string) => void;
  setKanbanTagFilter: (value: string) => void;
  setKanbanPriorityFilter: (value: PriorityFilter) => void;
  setKanbanDueFilter: (value: DueFilter) => void;
  clearKanbanFilters: () => void;

  openAddTaskDialog: (context?: Partial<AddTaskDialogContext>) => void;
  closeAddTaskDialog: () => void;
  createTask: (input: TaskCreate) => Promise<void>;
  updateTaskByUuid: (uuid: string, patch: TaskPatch) => Promise<TaskDto | null>;
  markTaskDone: (uuid: string) => Promise<void>;
  markTaskUndone: (uuid: string) => Promise<void>;
  removeTask: (uuid: string) => Promise<void>;
  markTasksDoneBulk: (uuids: string[]) => Promise<void>;
  markTasksUndoneBulk: (uuids: string[]) => Promise<void>;
  removeTasksBulk: (uuids: string[]) => Promise<void>;

  setActiveKanbanBoard: (boardId: string | null) => void;
  createKanbanBoard: (requestedName: string) => void;
  renameActiveKanbanBoard: (requestedName: string) => void;
  deleteActiveKanbanBoard: () => Promise<void>;
  toggleKanbanCompactCards: () => void;
  setDraggingKanbanTask: (taskId: string | null) => void;
  setDragOverKanbanLane: (lane: string | null) => void;
  moveKanbanTask: (taskId: string, lane: string) => Promise<void>;
  moveKanbanTaskToBoard: (taskId: string, boardId: string | null, lane?: string) => Promise<void>;

  setCalendarView: (view: "year" | "quarter" | "month" | "week" | "day") => void;
  setCalendarFocusDateIso: (iso: string) => void;
  shiftCalendarFocus: (step: number) => void;
  navigateCalendar: (iso: string, view?: "year" | "quarter" | "month" | "week" | "day") => void;
  setCalendarTaskFilter: (value: string) => void;

  openNewExternalCalendar: () => ExternalCalendarSource;
  saveExternalCalendarSource: (source: ExternalCalendarSource) => void;
  deleteExternalCalendarSource: (calendarId: string) => void;
  syncExternalCalendarSource: (calendarId: string) => Promise<void>;
  syncAllExternalCalendars: () => Promise<void>;
  importExternalCalendarFile: (file: File) => Promise<void>;

  openSettings: () => void;
  closeSettings: () => void;
  setDueNotificationsEnabled: (enabled: boolean) => void;
  setDuePreNotifyEnabled: (enabled: boolean) => void;
  setDuePreNotifyMinutes: (minutes: number) => void;
  requestDueNotificationPermission: () => Promise<void>;
  scanDueNotifications: () => void;
  clearCommandFailures: () => void;
}

const initialBoards = loadKanbanBoardsSafe();
const initialActiveBoardId = loadActiveBoardSafe(initialBoards);
const initialExternalCalendars = loadExternalCalendarsSafe();
const initialDueNotificationConfig = sanitizeDueNotificationConfig(
  loadNotificationSettings<DueNotificationConfig>(defaultDueNotificationConfig())
);
const initialDueNotificationSent = Array.from(loadNotificationSentRegistry());
const initialDueNotificationPermission = browserDueNotificationPermission();
let overdueCalendarSweepInFlight = false;

export const useAppStore = create<AppState>((set, get) => {
  setCommandFailureSink((record) => {
    set((state) => ({
      commandFailures: [record, ...state.commandFailures].slice(0, 30)
    }));
  });

  return {
  bootstrapped: false,
  activeTab: loadWorkspaceTab(),
  themeMode: loadThemeMode(),
  loading: false,
  error: null,
  tasks: [],
  selectedTaskId: null,
  addTaskDialogOpen: false,
  addTaskDialogContext: {
    boardId: null,
    lockBoardSelection: false,
    allowRecurrence: true
  },
  taskFilters: emptyTaskFilters(),
  kanbanFilters: {
    ...emptyTaskFilters(),
    status: "Pending"
  },
  runtimeConfig: null,
  tagSchema: null,
  tagColorMap: {},
  kanbanBoards: initialBoards,
  activeKanbanBoardId: initialActiveBoardId,
  kanbanCompactCards: loadKanbanCompactCards(),
  draggingKanbanTaskId: null,
  dragOverKanbanLane: null,
  calendarView: loadCalendarViewMode(),
  calendarFocusDateIso: calendarDateToIso(todayInTimezone("America/Mexico_City")),
  calendarTaskFilter: "__all__",
  externalCalendars: initialExternalCalendars,
  externalCalendarBusy: false,
  externalCalendarLastSync: null,
  settingsOpen: false,
  dueNotificationConfig: initialDueNotificationConfig,
  dueNotificationPermission: initialDueNotificationPermission,
  dueNotificationSent: initialDueNotificationSent,
  commandFailures: [],

  async bootstrap() {
    if (get().bootstrapped) {
      return;
    }
    set({ loading: true, error: null });
    logger.info("app.bootstrap.start", "bootstrapping React shell state");

    try {
      await healthCheck();
      const [tasks, runtimeConfig, tagSchema] = await Promise.all([
        listTasks(),
        loadConfigSnapshot(),
        loadTagSchemaSnapshot()
      ]);
      const effective = resolveCalendarConfig(runtimeConfig);
      const today = todayInTimezone(effective.timezone);
      const tagColorMap = buildTagColorMap(tagSchema);

      saveKanbanBoards(get().kanbanBoards);
      saveActiveKanbanBoardId(get().activeKanbanBoardId);
      saveExternalCalendars(get().externalCalendars);
      saveCalendarViewMode(get().calendarView);
      saveNotificationSettings(get().dueNotificationConfig);
      saveNotificationSentRegistry(new Set(get().dueNotificationSent));

      set({
        bootstrapped: true,
        loading: false,
        tasks,
        selectedTaskId: tasks[0]?.uuid ?? null,
        runtimeConfig,
        tagSchema,
        tagColorMap,
        calendarFocusDateIso: calendarDateToIso(today),
        dueNotificationPermission: browserDueNotificationPermission()
      });
      logger.info("app.bootstrap.done", `tasks=${tasks.length} timezone=${effective.timezone}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      logger.error("app.bootstrap.error", message);
      set({ loading: false, error: message });
    }
  },

  async loadTasks() {
    logger.debug("tasks.load.start", "loading latest task snapshot");
    set({ loading: true, error: null });
    try {
      const tasks = await listTasks();
      set((state) => {
        const selectedTaskId = state.selectedTaskId && tasks.some((task) => task.uuid === state.selectedTaskId)
          ? state.selectedTaskId
          : tasks[0]?.uuid ?? null;
        return {
          loading: false,
          tasks,
          selectedTaskId
        };
      });
      logger.debug("tasks.load.done", `tasks=${tasks.length}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
      logger.error("tasks.load.error", message);
    }
  },

  setActiveTab(tab) {
    saveWorkspaceTab(tab);
    set({ activeTab: tab });
    logger.info("tab.change", tab);
  },

  toggleTheme() {
    const nextMode = get().themeMode === "day" ? "night" : "day";
    saveThemeMode(nextMode);
    set({ themeMode: nextMode });
    logger.info("theme.toggle", nextMode);
  },

  selectTask(taskId) {
    set({ selectedTaskId: taskId });
  },

  setTaskSearchFilter(value) {
    set((state) => ({ taskFilters: { ...state.taskFilters, search: value } }));
  },
  setTaskStatusFilter(value) {
    set((state) => ({ taskFilters: { ...state.taskFilters, status: value } }));
  },
  setTaskProjectFilter(value) {
    set((state) => ({ taskFilters: { ...state.taskFilters, project: value } }));
  },
  setTaskTagFilter(value) {
    set((state) => ({ taskFilters: { ...state.taskFilters, tag: value } }));
  },
  setTaskPriorityFilter(value) {
    set((state) => ({ taskFilters: { ...state.taskFilters, priority: value } }));
  },
  setTaskDueFilter(value) {
    set((state) => ({ taskFilters: { ...state.taskFilters, due: value } }));
  },
  clearTaskFilters() {
    set({ taskFilters: emptyTaskFilters() });
  },

  setKanbanStatusFilter(value) {
    set((state) => ({ kanbanFilters: { ...state.kanbanFilters, status: value } }));
  },
  setKanbanProjectFilter(value) {
    set((state) => ({ kanbanFilters: { ...state.kanbanFilters, project: value } }));
  },
  setKanbanTagFilter(value) {
    set((state) => ({ kanbanFilters: { ...state.kanbanFilters, tag: value } }));
  },
  setKanbanPriorityFilter(value) {
    set((state) => ({ kanbanFilters: { ...state.kanbanFilters, priority: value } }));
  },
  setKanbanDueFilter(value) {
    set((state) => ({ kanbanFilters: { ...state.kanbanFilters, due: value } }));
  },
  clearKanbanFilters() {
    set({
      kanbanFilters: {
        ...emptyTaskFilters(),
        status: "Pending"
      }
    });
  },

  openAddTaskDialog(context) {
    const fallback = get().activeTab === "kanban" && get().activeKanbanBoardId
      ? {
          boardId: get().activeKanbanBoardId,
          lockBoardSelection: true,
          allowRecurrence: true
        }
      : {
          boardId: null,
          lockBoardSelection: false,
          allowRecurrence: true
        };

    const merged: AddTaskDialogContext = {
      boardId: context?.boardId ?? fallback.boardId,
      lockBoardSelection: context?.lockBoardSelection ?? fallback.lockBoardSelection,
      allowRecurrence: context?.allowRecurrence ?? fallback.allowRecurrence
    };

    set({
      addTaskDialogOpen: true,
      addTaskDialogContext: merged
    });
    logger.info("modal.add_task.open", JSON.stringify(merged));
  },

  closeAddTaskDialog() {
    set({
      addTaskDialogOpen: false,
      addTaskDialogContext: {
        boardId: null,
        lockBoardSelection: false,
        allowRecurrence: true
      }
    });
  },

  async createTask(input) {
    set({ loading: true, error: null });
    logger.info("task.create.start", `title=${input.title}`);
    try {
      const created = await addTask(input);
      set((state) => ({
        loading: false,
        addTaskDialogOpen: false,
        tasks: [created, ...state.tasks],
        selectedTaskId: created.uuid,
        addTaskDialogContext: {
          boardId: null,
          lockBoardSelection: false,
          allowRecurrence: true
        }
      }));
      logger.info("task.create.done", created.uuid);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
      logger.error("task.create.error", message);
    }
  },

  async updateTaskByUuid(uuid, patch) {
    set({ loading: true, error: null });
    logger.debug("task.update.start", uuid);
    try {
      const updated = await updateTask({ uuid, patch });
      set((state) => ({
        loading: false,
        tasks: state.tasks.map((task) => (task.uuid === uuid ? updated : task))
      }));
      logger.debug("task.update.done", uuid);
      return updated;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
      logger.error("task.update.error", `${uuid}: ${message}`);
      return null;
    }
  },

  async markTaskDone(uuid) {
    const task = get().tasks.find((entry) => entry.uuid === uuid);
    if (!task) {
      return;
    }
    if (!(task.status === "Pending" || task.status === "Waiting")) {
      return;
    }
    if (!canManuallyCompleteTask(task, Date.now())) {
      const message = "Calendar events can only be completed after their due time has passed.";
      set({ error: message });
      logger.warn("task.done.blocked", `${uuid}: calendar task due date not reached`);
      return;
    }

    set({ loading: true, error: null });
    logger.info("task.done.start", uuid);
    try {
      const updated = await doneTask(uuid);
      set((state) => ({
        loading: false,
        tasks: state.tasks.map((task) => (task.uuid === uuid ? updated : task))
      }));
      logger.info("task.done.done", uuid);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
      logger.error("task.done.error", `${uuid}: ${message}`);
    }
  },

  async markTaskUndone(uuid) {
    const task = get().tasks.find((entry) => entry.uuid === uuid);
    if (!task || task.status !== "Completed") {
      return;
    }

    set({ loading: true, error: null });
    logger.info("task.uncomplete.start", uuid);
    try {
      const updated = await uncompleteTask(uuid);
      set((state) => ({
        loading: false,
        tasks: state.tasks.map((entry) => (entry.uuid === uuid ? updated : entry))
      }));
      logger.info("task.uncomplete.done", uuid);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
      logger.error("task.uncomplete.error", `${uuid}: ${message}`);
    }
  },

  async removeTask(uuid) {
    set({ loading: true, error: null });
    logger.info("task.delete.start", uuid);
    try {
      await deleteTask(uuid);
      set((state) => {
        const nextTasks = state.tasks.filter((task) => task.uuid !== uuid);
        return {
          loading: false,
          tasks: nextTasks,
          selectedTaskId: state.selectedTaskId === uuid ? nextTasks[0]?.uuid ?? null : state.selectedTaskId
        };
      });
      logger.info("task.delete.done", uuid);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
      logger.error("task.delete.error", `${uuid}: ${message}`);
    }
  },

  async markTasksDoneBulk(uuids) {
    const targetIds = [...new Set(uuids)];
    if (targetIds.length === 0) {
      return;
    }

    const nowMs = Date.now();
    const taskById = new Map(get().tasks.map((task) => [task.uuid, task] as const));
    const eligible: string[] = [];
    const blockedCalendar: string[] = [];
    for (const uuid of targetIds) {
      const task = taskById.get(uuid);
      if (!task) {
        continue;
      }
      if (!(task.status === "Pending" || task.status === "Waiting")) {
        continue;
      }
      if (!canManuallyCompleteTask(task, nowMs)) {
        blockedCalendar.push(uuid);
        continue;
      }
      eligible.push(uuid);
    }

    if (eligible.length === 0) {
      if (blockedCalendar.length > 0) {
        const message = `Blocked ${blockedCalendar.length} calendar task(s): due time has not passed yet.`;
        set({ error: message });
      }
      return;
    }

    set({ loading: true, error: null });
    logger.info("task.done.bulk.start", `count=${eligible.length}`);

    const updatedById = new Map<string, TaskDto>();
    const failed: string[] = [];
    for (const uuid of eligible) {
      try {
        const updated = await doneTask(uuid);
        updatedById.set(uuid, updated);
      } catch (error) {
        failed.push(uuid);
        logger.warn("task.done.bulk.item_error", `${uuid}: ${String(error)}`);
      }
    }

    const failureMessage = failed.length > 0 ? `Failed to complete ${failed.length} task(s).` : null;
    const blockedMessage = blockedCalendar.length > 0 ? `Blocked ${blockedCalendar.length} calendar task(s) before due time.` : null;
    const errorMessage = [failureMessage, blockedMessage].filter((entry) => Boolean(entry)).join(" ");
    set((state) => ({
      loading: false,
      error: errorMessage || null,
      tasks: state.tasks.map((task) => updatedById.get(task.uuid) ?? task)
    }));
    logger.info(
      "task.done.bulk.done",
      `completed=${updatedById.size} failed=${failed.length} blocked=${blockedCalendar.length}`
    );
  },

  async markTasksUndoneBulk(uuids) {
    const targetIds = [...new Set(uuids)];
    if (targetIds.length === 0) {
      return;
    }

    const taskById = new Map(get().tasks.map((task) => [task.uuid, task] as const));
    const eligible = targetIds.filter((uuid) => taskById.get(uuid)?.status === "Completed");
    if (eligible.length === 0) {
      return;
    }

    set({ loading: true, error: null });
    logger.info("task.uncomplete.bulk.start", `count=${eligible.length}`);

    const updatedById = new Map<string, TaskDto>();
    const failed: string[] = [];
    for (const uuid of eligible) {
      try {
        const updated = await uncompleteTask(uuid);
        updatedById.set(uuid, updated);
      } catch (error) {
        failed.push(uuid);
        logger.warn("task.uncomplete.bulk.item_error", `${uuid}: ${String(error)}`);
      }
    }

    set((state) => ({
      loading: false,
      error: failed.length > 0 ? `Failed to uncomplete ${failed.length} task(s).` : null,
      tasks: state.tasks.map((task) => updatedById.get(task.uuid) ?? task)
    }));
    logger.info(
      "task.uncomplete.bulk.done",
      `reopened=${updatedById.size} failed=${failed.length}`
    );
  },

  async removeTasksBulk(uuids) {
    const targetIds = [...new Set(uuids)];
    if (targetIds.length === 0) {
      return;
    }

    set({ loading: true, error: null });
    logger.info("task.delete.bulk.start", `count=${targetIds.length}`);

    const deleted = new Set<string>();
    const failed: string[] = [];
    for (const uuid of targetIds) {
      try {
        await deleteTask(uuid);
        deleted.add(uuid);
      } catch (error) {
        failed.push(uuid);
        logger.warn("task.delete.bulk.item_error", `${uuid}: ${String(error)}`);
      }
    }

    set((state) => {
      const nextTasks = state.tasks.filter((task) => !deleted.has(task.uuid));
      const selectedTaskId = state.selectedTaskId && deleted.has(state.selectedTaskId)
        ? nextTasks[0]?.uuid ?? null
        : state.selectedTaskId;
      return {
        loading: false,
        error: failed.length > 0 ? `Failed to delete ${failed.length} task(s).` : state.error,
        tasks: nextTasks,
        selectedTaskId
      };
    });
    logger.info(
      "task.delete.bulk.done",
      `deleted=${deleted.size} failed=${failed.length}`
    );
  },

  setActiveKanbanBoard(boardId) {
    saveActiveKanbanBoardId(boardId);
    set({ activeKanbanBoardId: boardId });
    logger.info("kanban.board.select", boardId ?? "(none)");
  },

  createKanbanBoard(requestedName) {
    const boards = get().kanbanBoards;
    const name = makeUniqueBoardName(boards, requestedName);
    if (!name.trim()) {
      return;
    }
    const board = {
      id: crypto.randomUUID(),
      name,
      color: nextBoardColor(boards)
    };
    const nextBoards = [...boards, board];
    saveKanbanBoards(nextBoards);
    saveActiveKanbanBoardId(board.id);
    set({
      kanbanBoards: nextBoards,
      activeKanbanBoardId: board.id
    });
    logger.info("kanban.board.create", `${board.id}:${board.name}`);
  },

  renameActiveKanbanBoard(requestedName) {
    const activeId = get().activeKanbanBoardId;
    if (!activeId) {
      return;
    }
    const boards = get().kanbanBoards;
    const uniqueName = makeUniqueBoardName(boards, requestedName, activeId);
    const nextBoards = boards.map((board) => (board.id === activeId ? { ...board, name: uniqueName } : board));
    saveKanbanBoards(nextBoards);
    set({ kanbanBoards: nextBoards });
    logger.info("kanban.board.rename", `${activeId}:${uniqueName}`);
  },

  async deleteActiveKanbanBoard() {
    const activeId = get().activeKanbanBoardId;
    if (!activeId) {
      return;
    }
    const boards = get().kanbanBoards;
    const nextBoards = boards.filter((board) => board.id !== activeId);
    const nextActive = nextBoards[0]?.id ?? null;

    saveKanbanBoards(nextBoards);
    saveActiveKanbanBoardId(nextActive);
    set({
      kanbanBoards: nextBoards,
      activeKanbanBoardId: nextActive
    });
    logger.warn("kanban.board.delete", activeId);

    const affectedTasks = get().tasks.filter((task) => {
      const editable = task.status === "Pending" || task.status === "Waiting";
      return editable && taskHasTagValue(task.tags, BOARD_TAG_KEY, activeId);
    });

    const updatedById = new Map<string, TaskDto>();
    for (const task of affectedTasks) {
      const nextTags = [...task.tags];
      removeTagsForKey(nextTags, BOARD_TAG_KEY);
      try {
        const updated = await updateTask({ uuid: task.uuid, patch: { tags: nextTags } });
        updatedById.set(task.uuid, updated);
      } catch (error) {
        logger.warn("kanban.board.delete.cleanup", `${task.uuid}: ${String(error)}`);
      }
    }

    if (updatedById.size > 0) {
      set((state) => ({
        tasks: state.tasks.map((task) => updatedById.get(task.uuid) ?? task)
      }));
    }
  },

  toggleKanbanCompactCards() {
    const next = !get().kanbanCompactCards;
    saveKanbanCompactCards(next);
    set({ kanbanCompactCards: next });
  },

  setDraggingKanbanTask(taskId) {
    set({ draggingKanbanTaskId: taskId });
  },

  setDragOverKanbanLane(lane) {
    set({ dragOverKanbanLane: lane });
  },

  async moveKanbanTask(taskId, lane) {
    set({ draggingKanbanTaskId: null, dragOverKanbanLane: null });
    const task = get().tasks.find((entry) => entry.uuid === taskId);
    if (!task) {
      return;
    }
    if (!(task.status === "Pending" || task.status === "Waiting")) {
      return;
    }

    const columns = kanbanColumnsFromSchema(get().tagSchema);
    const fallbackLane = defaultKanbanLane(get().tagSchema);
    const targetLane = columns.includes(lane) ? lane : fallbackLane;
    const nextTags = tagsForKanbanMove(task.tags, targetLane);

    logger.info("kanban.task.move", `${taskId} -> ${targetLane}`);
    await get().updateTaskByUuid(taskId, { tags: nextTags });
  },

  async moveKanbanTaskToBoard(taskId, boardId, lane) {
    const task = get().tasks.find((entry) => entry.uuid === taskId);
    if (!task) {
      return;
    }
    if (!(task.status === "Pending" || task.status === "Waiting")) {
      return;
    }

    const columns = kanbanColumnsFromSchema(get().tagSchema);
    const fallbackLane = defaultKanbanLane(get().tagSchema);
    const targetLane = lane && columns.includes(lane) ? lane : fallbackLane;
    const nextTags = tagsForKanbanMove(task.tags, targetLane, boardId);
    logger.info("kanban.task.move_board", `${taskId} -> board=${boardId ?? "(none)"} lane=${targetLane}`);
    await get().updateTaskByUuid(taskId, { tags: nextTags });
  },

  setCalendarView(view) {
    saveCalendarViewMode(view);
    set({ calendarView: view });
  },

  setCalendarFocusDateIso(iso) {
    set({ calendarFocusDateIso: iso });
  },

  shiftCalendarFocus(step) {
    const state = get();
    const focus = calendarDateFromIso(state.calendarFocusDateIso);
    const effective = resolveCalendarConfig(state.runtimeConfig);
    const shifted = shiftFocusDate(focus, state.calendarView, step, effective.policies.week_start);
    set({ calendarFocusDateIso: calendarDateToIso(shifted) });
  },

  navigateCalendar(iso, view) {
    const next = view ?? get().calendarView;
    saveCalendarViewMode(next);
    set({ calendarFocusDateIso: iso, calendarView: next });
  },

  setCalendarTaskFilter(value) {
    set({ calendarTaskFilter: value });
  },

  openNewExternalCalendar() {
    return newExternalCalendarSource(get().externalCalendars);
  },

  saveExternalCalendarSource(source) {
    const current = get().externalCalendars;
    const existingIndex = current.findIndex((entry) => entry.id === source.id);
    const normalizedSource: ExternalCalendarSource = {
      ...source,
      imported_ics_file: source.imported_ics_file || source.location.trim().toLowerCase().startsWith("file://"),
      refresh_minutes: source.imported_ics_file ? 0 : source.refresh_minutes
    };

    const next = existingIndex >= 0
      ? current.map((entry) => (entry.id === source.id ? normalizedSource : entry))
      : [...current, normalizedSource];
    const deduped = assignUniqueExternalCalendarColors(next);
    saveExternalCalendars(deduped);
    set({ externalCalendars: deduped });
    logger.info("external_calendar.save", `${source.id}:${source.name}`);
  },

  deleteExternalCalendarSource(calendarId) {
    const next = get().externalCalendars.filter((source) => source.id !== calendarId);
    saveExternalCalendars(next);
    set({ externalCalendars: next });
    logger.warn("external_calendar.delete", calendarId);
  },

  async syncExternalCalendarSource(calendarId) {
    const source = get().externalCalendars.find((entry) => entry.id === calendarId);
    if (!source) {
      return;
    }
    if (source.imported_ics_file) {
      set({ externalCalendarLastSync: `Sync skipped for ${source.name}: imported ICS snapshots are refreshed via import.` });
      return;
    }

    set({ externalCalendarBusy: true, error: null });
    logger.info("external_calendar.sync.start", `${source.id}:${source.name}`);
    try {
      const result = await syncExternalCalendar(source);
      await get().loadTasks();
      set({
        externalCalendarBusy: false,
        externalCalendarLastSync: `Synced ${source.name}: +${result.created} / ~${result.updated} / -${result.deleted}`
      });
      logger.info("external_calendar.sync.done", `${source.id} created=${result.created} updated=${result.updated} deleted=${result.deleted}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({
        externalCalendarBusy: false,
        error: message,
        externalCalendarLastSync: `Sync failed for ${source.name}: ${message}`
      });
      logger.error("external_calendar.sync.error", `${source.id}: ${message}`);
    }
  },

  async syncAllExternalCalendars() {
    const sources = get().externalCalendars.filter((source) => source.enabled && !source.imported_ics_file && source.refresh_minutes > 0);
    if (sources.length === 0) {
      set({ externalCalendarLastSync: "No enabled remote calendars configured for auto sync." });
      return;
    }

    set({ externalCalendarBusy: true, error: null });
    logger.info("external_calendar.sync_all.start", `sources=${sources.length}`);

    let created = 0;
    let updated = 0;
    let deleted = 0;
    let failed = 0;
    for (const source of sources) {
      try {
        const result = await syncExternalCalendar(source);
        created += result.created;
        updated += result.updated;
        deleted += result.deleted;
      } catch (error) {
        failed += 1;
        logger.error("external_calendar.sync_all.error", `${source.id}: ${String(error)}`);
      }
    }

    await get().loadTasks();
    set({
      externalCalendarBusy: false,
      externalCalendarLastSync: `Sync all complete: +${created} / ~${updated} / -${deleted}${failed > 0 ? ` (failures=${failed})` : ""}`
    });
    logger.info("external_calendar.sync_all.done", `created=${created} updated=${updated} deleted=${deleted} failed=${failed}`);
  },

  async importExternalCalendarFile(file) {
    set({ externalCalendarBusy: true, error: null });
    logger.info("external_calendar.import.start", file.name);

    try {
      const icsText = await file.text();
      if (!icsText.trim()) {
        throw new Error("ICS file is empty");
      }

      const baseName = file.name.replace(/\.ics$/i, "").trim() || "Imported Calendar";
      const source = newExternalCalendarSource(get().externalCalendars);
      source.name = baseName;
      source.location = `file://${file.name}`;
      source.imported_ics_file = true;
      source.refresh_minutes = 0;
      source.enabled = true;

      const result = await importExternalCalendarIcs(source, icsText);
      const nextSources = assignUniqueExternalCalendarColors([...get().externalCalendars, source]);
      saveExternalCalendars(nextSources);
      await get().loadTasks();

      set({
        externalCalendars: nextSources,
        externalCalendarBusy: false,
        externalCalendarLastSync: `Imported ${source.name}: +${result.created} / ~${result.updated} / -${result.deleted}`
      });
      logger.info("external_calendar.import.done", `${source.id} events=${result.remote_events}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({
        externalCalendarBusy: false,
        error: message,
        externalCalendarLastSync: `Import failed for ${file.name}: ${message}`
      });
      logger.error("external_calendar.import.error", `${file.name}: ${message}`);
    }
  },

  openSettings() {
    set({
      settingsOpen: true,
      dueNotificationPermission: browserDueNotificationPermission()
    });
  },

  closeSettings() {
    set({ settingsOpen: false });
  },

  setDueNotificationsEnabled(enabled) {
    set((state) => {
      const next = sanitizeDueNotificationConfig({
        ...state.dueNotificationConfig,
        enabled,
        pre_notify_enabled: enabled ? state.dueNotificationConfig.pre_notify_enabled : false
      });
      saveNotificationSettings(next);
      logger.info("settings.notifications.enabled", String(next.enabled));
      return {
        dueNotificationConfig: next
      };
    });
  },

  setDuePreNotifyEnabled(enabled) {
    set((state) => {
      const next = sanitizeDueNotificationConfig({
        ...state.dueNotificationConfig,
        pre_notify_enabled: enabled
      });
      saveNotificationSettings(next);
      logger.info("settings.notifications.pre_enabled", String(next.pre_notify_enabled));
      return {
        dueNotificationConfig: next
      };
    });
  },

  setDuePreNotifyMinutes(minutes) {
    set((state) => {
      const next = sanitizeDueNotificationConfig({
        ...state.dueNotificationConfig,
        pre_notify_minutes: minutes
      });
      saveNotificationSettings(next);
      logger.info("settings.notifications.pre_minutes", String(next.pre_notify_minutes));
      return {
        dueNotificationConfig: next
      };
    });
  },

  async requestDueNotificationPermission() {
    const permission = await requestBrowserDueNotificationPermission();
    set({ dueNotificationPermission: permission });
    logger.info("settings.notifications.permission", permission);
  },

  scanDueNotifications() {
    const state = get();
    const nowMs = Date.now();

    if (!overdueCalendarSweepInFlight) {
      const overdueCalendarTasks = state.tasks.filter((task) => (
        (task.status === "Pending" || task.status === "Waiting")
        && isCalendarEventTask(task)
        && canManuallyCompleteTask(task, nowMs)
      ));
      const prematurelyCompletedCalendarTasks = state.tasks.filter((task) => (
        task.status === "Completed"
        && isCalendarEventTask(task)
        && !canManuallyCompleteTask(task, nowMs)
      ));

      if (overdueCalendarTasks.length > 0 || prematurelyCompletedCalendarTasks.length > 0) {
        overdueCalendarSweepInFlight = true;
        logger.info(
          "calendar.auto_sweep.start",
          `complete=${overdueCalendarTasks.length} reopen=${prematurelyCompletedCalendarTasks.length}`
        );
        void (async () => {
          try {
            const updatedById = new Map<string, TaskDto>();
            let completeFailed = 0;
            let reopenFailed = 0;

            for (const task of prematurelyCompletedCalendarTasks) {
              try {
                const updated = await uncompleteTask(task.uuid);
                updatedById.set(task.uuid, updated);
              } catch (error) {
                reopenFailed += 1;
                logger.warn("calendar.auto_reopen.error", `${task.uuid}: ${String(error)}`);
              }
            }

            for (const task of overdueCalendarTasks) {
              try {
                const updated = await doneTask(task.uuid);
                updatedById.set(task.uuid, updated);
              } catch (error) {
                completeFailed += 1;
                logger.warn("calendar.auto_complete.error", `${task.uuid}: ${String(error)}`);
              }
            }

            if (updatedById.size > 0) {
              set((current) => ({
                tasks: current.tasks.map((task) => updatedById.get(task.uuid) ?? task)
              }));
            }

            logger.info(
              "calendar.auto_sweep.done",
              `updated=${updatedById.size} complete_failed=${completeFailed} reopen_failed=${reopenFailed}`
            );
          } finally {
            overdueCalendarSweepInFlight = false;
          }
        })();
      }
    }

    const permission = browserDueNotificationPermission();
    if (permission !== state.dueNotificationPermission) {
      set({ dueNotificationPermission: permission });
    }
    if (permission !== "granted") {
      return;
    }

    const effective = resolveCalendarConfig(state.runtimeConfig);
    const sent = new Set(state.dueNotificationSent);
    const events = collectDueNotificationEvents(
      state.tasks,
      effective.timezone,
      state.dueNotificationConfig,
      sent,
      nowMs
    );
    if (events.length === 0) {
      return;
    }

    let changed = false;
    for (const event of events) {
      const emitted = emitDueNotification(event.title, event.body);
      if (!emitted) {
        logger.warn("notifications.emit.failed", event.key);
        continue;
      }
      sent.add(event.key);
      changed = true;
      logger.info("notifications.emit.ok", event.key);
    }

    if (changed) {
      const nextSent = Array.from(sent);
      saveNotificationSentRegistry(sent);
      set({ dueNotificationSent: nextSent });
      logger.debug(
        "notifications.sent_registry",
        `${DUE_NOTIFICATION_SENT_STORAGE_KEY} size=${nextSent.length}`
      );
    }
  },

  clearCommandFailures() {
    set({ commandFailures: [] });
  }
  };
});

export function useTaskViewData(): {
  visibleTasks: TaskDto[];
  projectFacets: Array<{ value: string; count: number }>;
  tagFacets: Array<{ value: string; count: number }>;
} {
  const tasks = useAppStore((state) => state.tasks);
  const filters = useAppStore((state) => state.taskFilters);

  return useMemo(() => {
    const visibleTasks = filterTasks(tasks, filters);
    const facets = buildTaskFacets(visibleTasks);
    return {
      visibleTasks,
      projectFacets: facets.projectFacets,
      tagFacets: facets.tagFacets
    };
  }, [tasks, filters]);
}

export function useSelectedTask(): TaskDto | null {
  const selectedTaskId = useAppStore((state) => state.selectedTaskId);
  const tasks = useAppStore((state) => state.tasks);
  if (!selectedTaskId) {
    return null;
  }
  return tasks.find((task) => task.uuid === selectedTaskId) ?? null;
}

export function useKanbanColumns(): string[] {
  const schema = useAppStore((state) => state.tagSchema);
  return kanbanColumnsFromSchema(schema);
}

export function useKanbanViewData(): {
  boardTasks: TaskDto[];
  visibleTasks: TaskDto[];
  projectFacets: Array<{ value: string; count: number }>;
  tagFacets: Array<{ value: string; count: number }>;
} {
  const tasks = useAppStore((state) => state.tasks);
  const activeBoardId = useAppStore((state) => state.activeKanbanBoardId);
  const filters = useAppStore((state) => state.kanbanFilters);

  return useMemo(() => {
    const boardTasks = tasks.filter((task) => {
      if (!activeBoardId) {
        return false;
      }
      const boardId = boardIdFromTaskTags(task.tags);
      return boardId === activeBoardId;
    });
    const visibleTasks = filterTasks(boardTasks, filters);
    const facets = buildTaskFacets(boardTasks);
    return {
      boardTasks,
      visibleTasks,
      projectFacets: facets.projectFacets,
      tagFacets: facets.tagFacets
    };
  }, [tasks, activeBoardId, filters]);
}

export function useBoardColorMap(): Record<string, string> {
  const boards = useAppStore((state) => state.kanbanBoards);
  const map: Record<string, string> = {};
  for (const board of boards) {
    map[board.id] = board.color;
  }
  return map;
}

export function useExternalCalendarColorMap(): Record<string, string> {
  const sources = useAppStore((state) => state.externalCalendars);
  const map: Record<string, string> = {};
  for (const source of sources) {
    map[source.id] = source.color;
  }
  return map;
}

export function useCalendarDueEntries() {
  const tasks = useAppStore((state) => state.tasks);
  const runtimeConfig = useAppStore((state) => state.runtimeConfig);
  const boardColors = useBoardColorMap();
  const calendarColors = useExternalCalendarColorMap();
  const config = resolveCalendarConfig(runtimeConfig);
  const entries = collectCalendarDueTasks(tasks, config, boardColors, calendarColors);
  return {
    config,
    entries
  };
}

export function buildTaskCreateWithTagSchema(
  input: Omit<TaskCreate, "tags"> & {
    selectedTags: string[];
    customTagInput: string;
    boardId: string | null;
    allowRecurrence: boolean;
    recurrence: RecurrenceDraft;
  },
  tagSchema: TagSchema | null
): TaskCreate {
  const lane = defaultKanbanLane(tagSchema);
  const boardTag = input.boardId ? `${BOARD_TAG_KEY}:${input.boardId}` : null;
  const tags = collectTagsForSubmit({
    selectedTags: input.selectedTags,
    customTagInput: input.customTagInput,
    boardTag,
    allowRecurrence: input.allowRecurrence,
    recurrence: input.recurrence,
    ensureKanbanLane: true,
    defaultKanbanLaneValue: lane
  });
  return {
    ...input,
    tags
  };
}

export function buildTaskUpdatePatchWithTagSchema(
  uuid: string,
  existingTask: TaskDto,
  customTagInput: string,
  boardId: string | null,
  recurrence: RecurrenceDraft,
  allowRecurrence: boolean,
  tagSchema: TagSchema | null
): { uuid: string; patch: TaskPatch } {
  const lane = defaultKanbanLane(tagSchema);
  const boardTag = boardId ? `${BOARD_TAG_KEY}:${boardId}` : null;
  const tags = collectTagsForSubmit({
    selectedTags: [...existingTask.tags],
    customTagInput,
    boardTag,
    allowRecurrence,
    recurrence,
    ensureKanbanLane: false,
    defaultKanbanLaneValue: lane
  });
  return {
    uuid,
    patch: {
      tags
    }
  };
}

export function addCustomTagsToList(tags: string[], customInput: string): string[] {
  const next = [...tags];
  for (const tag of splitTags(customInput)) {
    pushTagUnique(next, tag);
  }
  return next;
}
