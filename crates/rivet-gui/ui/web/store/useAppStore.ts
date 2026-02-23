import { create } from "zustand";

import { addTask, deleteTask, doneTask, healthCheck, listTasks, loadConfigSnapshot, loadTagSchemaSnapshot } from "../api/tauri";
import { logger } from "../lib/logger";
import type { RivetRuntimeConfig, TagSchema } from "../types/config";
import type { TaskCreate, TaskDto, TaskStatus } from "../types/core";

export type WorkspaceTab = "tasks" | "kanban" | "calendar";
export type ThemeMode = "day" | "night";
export type StatusFilter = "all" | TaskStatus;

interface TaskFilters {
  search: string;
  status: StatusFilter;
  project: string;
  tag: string;
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
  filters: TaskFilters;
  runtimeConfig: RivetRuntimeConfig | null;
  tagSchema: TagSchema | null;
  bootstrap: () => Promise<void>;
  loadTasks: () => Promise<void>;
  setActiveTab: (tab: WorkspaceTab) => void;
  toggleTheme: () => void;
  selectTask: (taskId: string | null) => void;
  setSearchFilter: (value: string) => void;
  setStatusFilter: (value: StatusFilter) => void;
  setProjectFilter: (value: string) => void;
  setTagFilter: (value: string) => void;
  clearFilters: () => void;
  openAddTaskDialog: () => void;
  closeAddTaskDialog: () => void;
  createTask: (input: TaskCreate) => Promise<void>;
  markTaskDone: (uuid: string) => Promise<void>;
  removeTask: (uuid: string) => Promise<void>;
}

const THEME_STORAGE_KEY = "rivet.theme";
const TAB_STORAGE_KEY = "rivet.workspace_tab";

function loadThemeMode(): ThemeMode {
  if (typeof window === "undefined") {
    return "day";
  }
  const raw = window.localStorage.getItem(THEME_STORAGE_KEY);
  return raw === "night" ? "night" : "day";
}

function saveThemeMode(mode: ThemeMode): void {
  if (typeof window === "undefined") {
    return;
  }
  window.localStorage.setItem(THEME_STORAGE_KEY, mode);
}

function loadWorkspaceTab(): WorkspaceTab {
  if (typeof window === "undefined") {
    return "tasks";
  }
  const raw = window.localStorage.getItem(TAB_STORAGE_KEY);
  if (raw === "kanban" || raw === "calendar") {
    return raw;
  }
  return "tasks";
}

function saveWorkspaceTab(tab: WorkspaceTab): void {
  if (typeof window === "undefined") {
    return;
  }
  window.localStorage.setItem(TAB_STORAGE_KEY, tab);
}

function filterTasks(tasks: TaskDto[], filters: TaskFilters): TaskDto[] {
  const query = filters.search.trim().toLowerCase();
  const project = filters.project.trim().toLowerCase();
  const tag = filters.tag.trim().toLowerCase();

  return tasks.filter((task) => {
    if (filters.status !== "all" && task.status !== filters.status) {
      return false;
    }

    if (query.length > 0) {
      const haystack = [task.title, task.description, task.project ?? "", task.tags.join(" ")].join(" ").toLowerCase();
      if (!haystack.includes(query)) {
        return false;
      }
    }

    if (project.length > 0 && (task.project ?? "").toLowerCase() !== project) {
      return false;
    }

    if (tag.length > 0 && !task.tags.some((value) => value.toLowerCase().includes(tag))) {
      return false;
    }

    return true;
  });
}

export const useAppStore = create<AppState>((set, get) => ({
  bootstrapped: false,
  activeTab: loadWorkspaceTab(),
  themeMode: loadThemeMode(),
  loading: false,
  error: null,
  tasks: [],
  selectedTaskId: null,
  addTaskDialogOpen: false,
  filters: {
    search: "",
    status: "all",
    project: "",
    tag: ""
  },
  runtimeConfig: null,
  tagSchema: null,
  async bootstrap() {
    if (get().bootstrapped) {
      return;
    }

    set({ loading: true, error: null });
    logger.info("app.bootstrap", "starting React workspace bootstrap");

    try {
      await healthCheck();
      const [tasks, runtimeConfig, tagSchema] = await Promise.all([listTasks(), loadConfigSnapshot(), loadTagSchemaSnapshot()]);
      set({
        bootstrapped: true,
        loading: false,
        tasks,
        selectedTaskId: tasks[0]?.uuid ?? null,
        runtimeConfig,
        tagSchema
      });
      logger.info("app.bootstrap", `bootstrap finished with ${tasks.length} tasks`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      logger.error("app.bootstrap", message);
      set({ loading: false, error: message });
    }
  },
  async loadTasks() {
    set({ loading: true, error: null });
    try {
      const tasks = await listTasks();
      set((state) => {
        const selectedTaskId = state.selectedTaskId && tasks.some((task) => task.uuid === state.selectedTaskId)
          ? state.selectedTaskId
          : tasks[0]?.uuid ?? null;
        return { loading: false, tasks, selectedTaskId };
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },
  setActiveTab(tab) {
    saveWorkspaceTab(tab);
    set({ activeTab: tab });
  },
  toggleTheme() {
    const next = get().themeMode === "day" ? "night" : "day";
    saveThemeMode(next);
    set({ themeMode: next });
  },
  selectTask(taskId) {
    set({ selectedTaskId: taskId });
  },
  setSearchFilter(value) {
    set((state) => ({
      filters: {
        ...state.filters,
        search: value
      }
    }));
  },
  setStatusFilter(value) {
    set((state) => ({
      filters: {
        ...state.filters,
        status: value
      }
    }));
  },
  setProjectFilter(value) {
    set((state) => ({
      filters: {
        ...state.filters,
        project: value
      }
    }));
  },
  setTagFilter(value) {
    set((state) => ({
      filters: {
        ...state.filters,
        tag: value
      }
    }));
  },
  clearFilters() {
    set((state) => ({
      filters: {
        ...state.filters,
        search: "",
        status: "all",
        project: "",
        tag: ""
      }
    }));
  },
  openAddTaskDialog() {
    set({ addTaskDialogOpen: true });
  },
  closeAddTaskDialog() {
    set({ addTaskDialogOpen: false });
  },
  async createTask(input) {
    set({ loading: true, error: null });
    try {
      const created = await addTask(input);
      set((state) => ({
        loading: false,
        addTaskDialogOpen: false,
        tasks: [created, ...state.tasks],
        selectedTaskId: created.uuid
      }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
      logger.error("task.create", message);
    }
  },
  async markTaskDone(uuid) {
    set({ loading: true, error: null });
    try {
      const updated = await doneTask(uuid);
      set((state) => ({
        loading: false,
        tasks: state.tasks.map((task) => (task.uuid === uuid ? updated : task))
      }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
      logger.error("task.done", message);
    }
  },
  async removeTask(uuid) {
    set({ loading: true, error: null });
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
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
      logger.error("task.delete", message);
    }
  }
}));

export function useFilteredTasks(): TaskDto[] {
  const tasks = useAppStore((state) => state.tasks);
  const filters = useAppStore((state) => state.filters);
  return filterTasks(tasks, filters);
}

export function useSelectedTask(): TaskDto | null {
  const selectedTaskId = useAppStore((state) => state.selectedTaskId);
  const tasks = useAppStore((state) => state.tasks);
  if (!selectedTaskId) {
    return null;
  }
  return tasks.find((task) => task.uuid === selectedTaskId) ?? null;
}
