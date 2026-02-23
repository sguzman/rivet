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
  uncompleteTaskMock: vi.fn(),
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
  uncompleteTask: mocks.uncompleteTaskMock,
  updateTask: mocks.updateTaskMock
}));

import { useAppStore } from "./useAppStore";

const initialState = useAppStore.getState();

function sampleTask(title: string, overrides: Partial<TaskDto> = {}): TaskDto {
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
    modified: new Date().toISOString(),
    ...overrides
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
    mocks.uncompleteTaskMock.mockReset();
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

  it("updates tasks via updateTaskByUuid and unwinds loading", async () => {
    const existing = sampleTask("Editable task", {
      description: "Before edit",
      tags: ["kanban:todo", "board:main"]
    });
    const updated = {
      ...existing,
      title: "Edited title",
      description: "After edit",
      tags: ["kanban:working", "board:main"],
      modified: new Date(Date.now() + 1_000).toISOString()
    };

    useAppStore.setState({
      tasks: [existing],
      selectedTaskId: existing.uuid
    });
    mocks.updateTaskMock.mockResolvedValueOnce(updated);

    const result = await useAppStore.getState().updateTaskByUuid(existing.uuid, {
      title: "Edited title",
      description: "After edit",
      tags: ["kanban:working", "board:main"]
    });

    expect(result).not.toBeNull();
    const current = useAppStore.getState();
    expect(current.loading).toBe(false);
    expect(current.error).toBeNull();
    expect(current.tasks[0]?.title).toBe("Edited title");
    expect(current.tasks[0]?.description).toBe("After edit");
    expect(current.tasks[0]?.tags).toEqual(["kanban:working", "board:main"]);
  });

  it("supports bulk completion and bulk deletion", async () => {
    const one = sampleTask("Bulk one");
    const two = sampleTask("Bulk two");
    const three = sampleTask("Bulk three");

    useAppStore.setState({
      tasks: [one, two, three],
      selectedTaskId: two.uuid
    });

    mocks.doneTaskMock.mockResolvedValueOnce({ ...one, status: "Completed" });
    mocks.doneTaskMock.mockResolvedValueOnce({ ...two, status: "Completed" });
    await useAppStore.getState().markTasksDoneBulk([one.uuid, two.uuid, two.uuid]);

    let current = useAppStore.getState();
    expect(current.loading).toBe(false);
    expect(current.tasks.find((task) => task.uuid === one.uuid)?.status).toBe("Completed");
    expect(current.tasks.find((task) => task.uuid === two.uuid)?.status).toBe("Completed");
    expect(mocks.doneTaskMock).toHaveBeenCalledTimes(2);

    mocks.deleteTaskMock.mockResolvedValueOnce(undefined);
    mocks.deleteTaskMock.mockRejectedValueOnce(new Error("delete failed"));
    await useAppStore.getState().removeTasksBulk([one.uuid, three.uuid]);

    current = useAppStore.getState();
    expect(current.loading).toBe(false);
    expect(current.tasks.some((task) => task.uuid === one.uuid)).toBe(false);
    expect(current.tasks.some((task) => task.uuid === three.uuid)).toBe(true);
    expect(current.error).toContain("Failed to delete");
  });

  it("moves a task between kanban boards and lanes with updated tags", async () => {
    const source = sampleTask("Kanban move", {
      tags: ["kanban:todo", "board:alpha"]
    });
    const moved = sampleTask("Kanban move", {
      uuid: source.uuid,
      tags: ["kanban:working", "board:beta"]
    });

    useAppStore.setState({
      tasks: [source],
      tagSchema: {
        version: 1,
        keys: [
          { id: "kanban", values: ["todo", "working", "finished"] }
        ]
      }
    });
    mocks.updateTaskMock.mockResolvedValueOnce(moved);

    await useAppStore.getState().moveKanbanTaskToBoard(source.uuid, "beta", "working");

    expect(mocks.updateTaskMock).toHaveBeenCalledTimes(1);
    expect(mocks.updateTaskMock.mock.calls[0]?.[0]).toMatchObject({
      uuid: source.uuid,
      patch: {
        tags: ["kanban:working", "board:beta"]
      }
    });
    const current = useAppStore.getState();
    expect(current.tasks[0]?.tags).toEqual(["kanban:working", "board:beta"]);
  });
});
