import { describe, expect, it } from "vitest";

import type { TaskDto } from "../types/core";
import type { TaskFilters } from "../types/ui";
import { buildTaskFacets, filterTasks } from "./selectors";

function makeTask(index: number): TaskDto {
  return {
    uuid: `task-${index}`,
    id: index,
    title: `Task ${index}`,
    description: index % 2 === 0 ? "alpha work" : "beta work",
    status: index % 5 === 0 ? "Completed" : "Pending",
    project: index % 3 === 0 ? "proj-a" : "proj-b",
    tags: [`area:${index % 2 === 0 ? "alpha" : "beta"}`, `kanban:${index % 3 === 0 ? "todo" : "working"}`],
    priority: index % 2 === 0 ? "High" : null,
    due: index % 4 === 0 ? "2026-03-01T10:00:00Z" : null,
    wait: null,
    scheduled: null,
    created: null,
    modified: null
  };
}

const baseFilters: TaskFilters = {
  search: "",
  status: "all",
  project: "",
  tag: "",
  priority: "all",
  due: "all"
};

describe("store selectors", () => {
  it("filters tasks by status/project/tag", () => {
    const tasks = Array.from({ length: 30 }).map((_, index) => makeTask(index));
    const filtered = filterTasks(tasks, {
      ...baseFilters,
      status: "Pending",
      project: "proj-a",
      tag: "area:alpha"
    });
    expect(filtered.length).toBeGreaterThan(0);
    expect(filtered.every((task) => task.status === "Pending")).toBe(true);
    expect(filtered.every((task) => task.project === "proj-a")).toBe(true);
    expect(filtered.every((task) => task.tags.includes("area:alpha"))).toBe(true);
  });

  it("builds project and tag facets", () => {
    const tasks = Array.from({ length: 12 }).map((_, index) => makeTask(index));
    const facets = buildTaskFacets(tasks);
    expect(facets.projectFacets.length).toBeGreaterThan(0);
    expect(facets.tagFacets.some((entry) => entry.value.startsWith("area:"))).toBe(true);
  });

  it("handles large datasets within practical runtime budget", () => {
    const tasks = Array.from({ length: 12_000 }).map((_, index) => makeTask(index));
    const startedAt = performance.now();
    const filtered = filterTasks(tasks, {
      ...baseFilters,
      status: "Pending",
      search: "alpha",
      due: "has_due"
    });
    const elapsedMs = performance.now() - startedAt;

    expect(filtered.length).toBeGreaterThan(0);
    expect(elapsedMs).toBeLessThan(1_500);
  });
});
