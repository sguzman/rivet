import { beforeEach, describe, expect, it, vi } from "vitest";

import type { TaskCreate, TaskDto } from "../types/core";

const mocks = vi.hoisted(() => ({
  addTaskMock: vi.fn(),
  deleteTaskMock: vi.fn(),
  doneTaskMock: vi.fn(),
  healthCheckMock: vi.fn(),
  importExternalCalendarIcsMock: vi.fn(),
  listTasksMock: vi.fn(),
  loadConfigSnapshotMock: vi.fn(),
  loadTagSchemaSnapshotMock: vi.fn(),
  setCommandFailureSinkMock: vi.fn(),
  syncExternalCalendarMock: vi.fn(),
  updateTaskMock: vi.fn()
}));

vi.mock("../api/tauri", () => ({
  addTask: mocks.addTaskMock,
  deleteTask: mocks.deleteTaskMock,
  doneTask: mocks.doneTaskMock,
  healthCheck: mocks.healthCheckMock,
  importExternalCalendarIcs: mocks.importExternalCalendarIcsMock,
  listTasks: mocks.listTasksMock,
  loadConfigSnapshot: mocks.loadConfigSnapshotMock,
  loadTagSchemaSnapshot: mocks.loadTagSchemaSnapshotMock,
  setCommandFailureSink: mocks.setCommandFailureSinkMock,
  syncExternalCalendar: mocks.syncExternalCalendarMock,
  updateTask: mocks.updateTaskMock
}));

import { useAppStore } from "./useAppStore";

const initialState = useAppStore.getState();

function sampleTask(title: string): TaskDto {
  return {
    uuid: `uuid-${title.toLowerCase().replace(/\s+/g, "-")}`,
    id: null,
    title,
    description: "",
    status: "Pending",
    project: null,
    tags: ["kanban:todo"],
    priority: null,
    due: null,
    wait: null,
    scheduled: null,
    created: new Date().toISOString(),
    modified: new Date().toISOString()
  };
}

const createInput: TaskCreate = {
  title: "Create test task",
  description: "",
  project: null,
  tags: ["kanban:todo"],
  priority: null,
  due: null,
  wait: null,
  scheduled: null
};

describe("useAppStore modal and save regressions", () => {
  beforeEach(() => {
    mocks.addTaskMock.mockReset();
    mocks.deleteTaskMock.mockReset();
    mocks.doneTaskMock.mockReset();
    mocks.healthCheckMock.mockReset();
    mocks.importExternalCalendarIcsMock.mockReset();
    mocks.listTasksMock.mockReset();
    mocks.loadConfigSnapshotMock.mockReset();
    mocks.loadTagSchemaSnapshotMock.mockReset();
    mocks.syncExternalCalendarMock.mockReset();
    mocks.updateTaskMock.mockReset();

    useAppStore.setState(initialState, true);
  });

  it("opens and closes add-task modal without sticky state", () => {
    const state = useAppStore.getState();
    state.openAddTaskDialog({
      boardId: "board-a",
      lockBoardSelection: true,
      allowRecurrence: false
    });
    let current = useAppStore.getState();
    expect(current.addTaskDialogOpen).toBe(true);
    expect(current.addTaskDialogContext.boardId).toBe("board-a");

    current.closeAddTaskDialog();
    current = useAppStore.getState();
    expect(current.addTaskDialogOpen).toBe(false);
    expect(current.addTaskDialogContext.boardId).toBeNull();
    expect(current.addTaskDialogContext.lockBoardSelection).toBe(false);
    expect(current.addTaskDialogContext.allowRecurrence).toBe(true);
  });

  it("unwinds loading/error correctly on save failure and remains interactive", async () => {
    mocks.addTaskMock.mockRejectedValueOnce(new Error("save failed"));

    useAppStore.getState().openAddTaskDialog();
    await useAppStore.getState().createTask(createInput);

    const afterFailure = useAppStore.getState();
    expect(afterFailure.loading).toBe(false);
    expect(afterFailure.error).toContain("save failed");
    expect(afterFailure.addTaskDialogOpen).toBe(true);

    const previousTheme = afterFailure.themeMode;
    afterFailure.toggleTheme();
    expect(useAppStore.getState().themeMode).not.toBe(previousTheme);
    afterFailure.setActiveTab("calendar");
    expect(useAppStore.getState().activeTab).toBe("calendar");
  });

  it("closes modal and selects newly created task on successful save", async () => {
    const created = sampleTask("Created from modal");
    mocks.addTaskMock.mockResolvedValueOnce(created);

    useAppStore.getState().openAddTaskDialog();
    await useAppStore.getState().createTask({
      ...createInput,
      title: created.title
    });

    const current = useAppStore.getState();
    expect(current.addTaskDialogOpen).toBe(false);
    expect(current.loading).toBe(false);
    expect(current.error).toBeNull();
    expect(current.tasks[0]?.uuid).toBe(created.uuid);
    expect(current.selectedTaskId).toBe(created.uuid);
  });
});
