import type { TaskDto, TaskStatus } from "../types/core";
import type { DueFilter, PriorityFilter, StatusFilter, TaskFilters } from "../types/ui";

function compareText(haystack: string, needle: string): boolean {
  return haystack.toLowerCase().includes(needle.toLowerCase());
}

function statusMatches(taskStatus: TaskStatus, statusFilter: StatusFilter): boolean {
  if (statusFilter === "all") {
    return true;
  }
  return taskStatus === statusFilter;
}

function priorityMatches(task: TaskDto, priorityFilter: PriorityFilter): boolean {
  if (priorityFilter === "all") {
    return true;
  }
  if (priorityFilter === "none") {
    return task.priority === null;
  }
  return task.priority?.toLowerCase() === priorityFilter;
}

function dueMatches(task: TaskDto, dueFilter: DueFilter): boolean {
  if (dueFilter === "all") {
    return true;
  }
  if (dueFilter === "has_due") {
    return Boolean(task.due);
  }
  return !task.due;
}

export function matchesFilters(task: TaskDto, filters: TaskFilters): boolean {
  if (!statusMatches(task.status, filters.status)) {
    return false;
  }
  if (!priorityMatches(task, filters.priority)) {
    return false;
  }
  if (!dueMatches(task, filters.due)) {
    return false;
  }

  const search = filters.search.trim();
  if (search.length > 0) {
    const haystack = [task.title, task.description, task.project ?? "", task.tags.join(" ")].join(" ");
    if (!compareText(haystack, search)) {
      return false;
    }
  }

  const project = filters.project.trim();
  if (project.length > 0) {
    if (!compareText(task.project ?? "", project)) {
      return false;
    }
  }

  const tag = filters.tag.trim();
  if (tag.length > 0) {
    if (!task.tags.some((entry) => compareText(entry, tag))) {
      return false;
    }
  }

  return true;
}

export function facetCounts(items: string[]): Array<{ value: string; count: number }> {
  const map = new Map<string, number>();
  for (const item of items) {
    if (!item.trim()) {
      continue;
    }
    map.set(item, (map.get(item) ?? 0) + 1);
  }
  return [...map.entries()]
    .map(([value, count]) => ({ value, count }))
    .sort((a, b) => a.value.localeCompare(b.value));
}

export interface TaskFacetBundle {
  projectFacets: Array<{ value: string; count: number }>;
  tagFacets: Array<{ value: string; count: number }>;
}

export function buildTaskFacets(tasks: TaskDto[]): TaskFacetBundle {
  const projects: string[] = [];
  const tags: string[] = [];

  for (const task of tasks) {
    projects.push(task.project ?? "");
    for (const tag of task.tags) {
      tags.push(tag);
    }
  }

  return {
    projectFacets: facetCounts(projects),
    tagFacets: facetCounts(tags)
  };
}

export function filterTasks(tasks: TaskDto[], filters: TaskFilters): TaskDto[] {
  return tasks.filter((task) => matchesFilters(task, filters));
}
