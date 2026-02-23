import { describe, expect, it } from "vitest";

import { tagsForKanbanMove } from "./tags";

describe("tagsForKanbanMove", () => {
  it("updates lane while preserving board when boardId is omitted", () => {
    const input = ["board:alpha", "kanban:todo", "area:ui"];
    const next = tagsForKanbanMove(input, "working");
    expect(next).toContain("board:alpha");
    expect(next).toContain("kanban:working");
    expect(next).not.toContain("kanban:todo");
    expect(next).toContain("area:ui");
  });

  it("updates both lane and board when boardId is provided", () => {
    const input = ["board:alpha", "kanban:todo", "area:ui"];
    const next = tagsForKanbanMove(input, "finished", "beta");
    expect(next).toContain("board:beta");
    expect(next).not.toContain("board:alpha");
    expect(next).toContain("kanban:finished");
    expect(next).not.toContain("kanban:todo");
  });

  it("clears board tag when null boardId is provided", () => {
    const input = ["board:alpha", "kanban:todo", "area:ui"];
    const next = tagsForKanbanMove(input, "todo", null);
    expect(next.some((tag) => tag.startsWith("board:"))).toBe(false);
    expect(next).toContain("kanban:todo");
  });
});
