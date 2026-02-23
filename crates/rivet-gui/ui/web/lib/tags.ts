import type { TagKey, TagSchema } from "../types/config";
import type { RecurrenceDraft } from "../types/ui";

export const KANBAN_TAG_KEY = "kanban";
export const BOARD_TAG_KEY = "board";
export const RECUR_TAG_KEY = "recur";
export const RECUR_TIME_TAG_KEY = "recur_time";
export const RECUR_DAYS_TAG_KEY = "recur_days";
export const RECUR_MONTHS_TAG_KEY = "recur_months";
export const RECUR_MONTH_DAY_TAG_KEY = "recur_day";
export const CAL_SOURCE_TAG_KEY = "cal_source";
export const CAL_COLOR_TAG_KEY = "cal_color";
export const CALENDAR_UNAFFILIATED_COLOR = "#7f8691";

export const WEEKDAY_KEYS = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"] as const;
export const MONTH_KEYS = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"] as const;

export type WeekdayKey = (typeof WEEKDAY_KEYS)[number];
export type MonthKey = (typeof MONTH_KEYS)[number];

export function splitTags(text: string): string[] {
  return text
    .split(/\s+/)
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);
}

export function pushTagUnique(tags: string[], tag: string): boolean {
  const normalized = tag.trim();
  if (normalized.length === 0) {
    return false;
  }
  if (tags.some((entry) => entry === normalized)) {
    return false;
  }
  tags.push(normalized);
  return true;
}

export function removeTagsForKey(tags: string[], key: string): void {
  for (let index = tags.length - 1; index >= 0; index -= 1) {
    const tag = tags[index];
    const [entryKey] = splitTag(tag);
    if (entryKey === key) {
      tags.splice(index, 1);
    }
  }
}

export function splitTag(tag: string): [string | null, string | null] {
  const pair = tag.split(":");
  if (pair.length < 2) {
    return [null, null];
  }
  const key = pair[0]?.trim() ?? "";
  const value = pair.slice(1).join(":").trim();
  if (!key || !value) {
    return [null, null];
  }
  return [key, value];
}

export function firstTagValue(tags: string[], key: string): string | null {
  for (const tag of tags) {
    const [entryKey, entryValue] = splitTag(tag);
    if (entryKey === key && entryValue) {
      return entryValue;
    }
  }
  return null;
}

export function taskHasTagValue(tags: string[], key: string, value: string): boolean {
  return tags.some((tag) => {
    const [entryKey, entryValue] = splitTag(tag);
    return entryKey === key && entryValue === value;
  });
}

export function normalizeHexColor(value: string): string | null {
  const raw = value.trim().replace(/^#/, "");
  if (/^[0-9a-fA-F]{3}$/.test(raw)) {
    const expanded = raw
      .split("")
      .map((ch) => `${ch}${ch}`)
      .join("")
      .toLowerCase();
    return `#${expanded}`;
  }
  if (/^[0-9a-fA-F]{6}$/.test(raw)) {
    return `#${raw.toLowerCase()}`;
  }
  return null;
}

export function normalizeMarkerColor(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) {
    return CALENDAR_UNAFFILIATED_COLOR;
  }
  return normalizeHexColor(trimmed) ?? trimmed;
}

export function deterministicTagKeyColor(key: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < key.length; index += 1) {
    hash ^= key.charCodeAt(index);
    hash = (hash * 16777619) >>> 0;
  }
  const hue = hash % 360;
  return `hsl(${hue} 72% 54%)`;
}

export function colorForTagKey(key: TagKey): string {
  const explicit = key.color?.trim();
  if (explicit) {
    return explicit;
  }
  return deterministicTagKeyColor(key.id);
}

export function buildTagColorMap(schema: TagSchema | null): Record<string, string> {
  const map: Record<string, string> = {};
  for (const key of schema?.keys ?? []) {
    if (!key.id?.trim()) {
      continue;
    }
    map[key.id] = colorForTagKey(key);
  }
  return map;
}

export function tagColorStyle(tag: string, schema: TagSchema | null, colorMap: Record<string, string>): string {
  const [key] = splitTag(tag);
  if (!key) {
    return "";
  }
  const fromSchema = schema?.keys?.find((entry) => entry.id === key)?.color?.trim();
  const color = fromSchema || colorMap[key] || deterministicTagKeyColor(key);
  return color;
}

export function defaultKanbanLane(schema: TagSchema | null): string {
  const laneKey = schema?.keys?.find((entry) => entry.id === KANBAN_TAG_KEY);
  const first = laneKey?.values?.find((entry) => entry.trim().length > 0)?.trim();
  return first || "todo";
}

export function kanbanColumnsFromSchema(schema: TagSchema | null): string[] {
  const laneKey = schema?.keys?.find((entry) => entry.id === KANBAN_TAG_KEY);
  const values = (laneKey?.values ?? []).map((entry) => entry.trim()).filter((entry) => entry.length > 0);
  if (values.length === 0) {
    return ["todo", "working", "finished"];
  }
  return values;
}

export function normalizeRecurrencePattern(value: string): RecurrenceDraft["pattern"] {
  const normalized = value.trim().toLowerCase();
  if (
    normalized === "daily" ||
    normalized === "weekly" ||
    normalized === "months" ||
    normalized === "monthly" ||
    normalized === "yearly"
  ) {
    return normalized;
  }
  return "none";
}

export function appendRecurrenceTags(tags: string[], recurrence: RecurrenceDraft): void {
  removeTagsForKey(tags, RECUR_TAG_KEY);
  removeTagsForKey(tags, RECUR_TIME_TAG_KEY);
  removeTagsForKey(tags, RECUR_DAYS_TAG_KEY);
  removeTagsForKey(tags, RECUR_MONTHS_TAG_KEY);
  removeTagsForKey(tags, RECUR_MONTH_DAY_TAG_KEY);

  const pattern = normalizeRecurrencePattern(recurrence.pattern);
  if (pattern === "none") {
    return;
  }

  pushTagUnique(tags, `${RECUR_TAG_KEY}:${pattern}`);
  const recurrenceTime = recurrence.time.trim();
  if (recurrenceTime.length > 0) {
    pushTagUnique(tags, `${RECUR_TIME_TAG_KEY}:${recurrenceTime}`);
  }

  if (pattern === "weekly") {
    const days = recurrence.days
      .map((day) => day.trim().toLowerCase())
      .filter((day): day is WeekdayKey => WEEKDAY_KEYS.includes(day as WeekdayKey));
    if (days.length > 0) {
      pushTagUnique(tags, `${RECUR_DAYS_TAG_KEY}:${days.join(",")}`);
    }
  }

  if (pattern === "months" || pattern === "monthly" || pattern === "yearly") {
    const months = recurrence.months
      .map((month) => month.trim().toLowerCase())
      .filter((month): month is MonthKey => MONTH_KEYS.includes(month as MonthKey));
    if (months.length > 0) {
      pushTagUnique(tags, `${RECUR_MONTHS_TAG_KEY}:${months.join(",")}`);
    }
    const monthDay = recurrence.monthDay.trim();
    if (monthDay.length > 0) {
      pushTagUnique(tags, `${RECUR_MONTH_DAY_TAG_KEY}:${monthDay}`);
    }
  }
}

export function recurrenceFromTags(tags: string[]): RecurrenceDraft {
  const pattern = normalizeRecurrencePattern(firstTagValue(tags, RECUR_TAG_KEY) ?? "none");
  const time = firstTagValue(tags, RECUR_TIME_TAG_KEY) ?? "";
  const days = (firstTagValue(tags, RECUR_DAYS_TAG_KEY) ?? "")
    .split(",")
    .map((entry) => entry.trim().toLowerCase())
    .filter((entry): entry is WeekdayKey => WEEKDAY_KEYS.includes(entry as WeekdayKey));
  const months = (firstTagValue(tags, RECUR_MONTHS_TAG_KEY) ?? "")
    .split(",")
    .map((entry) => entry.trim().toLowerCase())
    .filter((entry): entry is MonthKey => MONTH_KEYS.includes(entry as MonthKey));
  const monthDay = firstTagValue(tags, RECUR_MONTH_DAY_TAG_KEY) ?? "";
  return {
    pattern,
    time,
    days,
    months,
    monthDay
  };
}

export function isSingleSelectKey(schema: TagSchema | null, keyId: string): boolean {
  const key = schema?.keys?.find((entry) => entry.id === keyId);
  return (key?.selection ?? "").toLowerCase() === "single";
}

export interface CollectSubmitTagsInput {
  selectedTags: string[];
  customTagInput: string;
  boardTag: string | null;
  allowRecurrence: boolean;
  recurrence: RecurrenceDraft;
  ensureKanbanLane: boolean;
  defaultKanbanLaneValue: string;
}

export function collectTagsForSubmit(input: CollectSubmitTagsInput): string[] {
  const tags = [...input.selectedTags];
  for (const tag of splitTags(input.customTagInput)) {
    pushTagUnique(tags, tag);
  }

  removeTagsForKey(tags, BOARD_TAG_KEY);
  if (input.boardTag) {
    pushTagUnique(tags, input.boardTag);
  }

  if (input.allowRecurrence) {
    appendRecurrenceTags(tags, input.recurrence);
  }

  if (input.ensureKanbanLane && !tags.some((tag) => tag.startsWith(`${KANBAN_TAG_KEY}:`))) {
    pushTagUnique(tags, `${KANBAN_TAG_KEY}:${input.defaultKanbanLaneValue}`);
  }
  return tags;
}

export function tagsForKanbanMove(
  tags: string[],
  lane: string,
  boardId?: string | null
): string[] {
  const next = [...tags];
  removeTagsForKey(next, KANBAN_TAG_KEY);
  pushTagUnique(next, `${KANBAN_TAG_KEY}:${lane}`);

  // `undefined` means keep board unchanged; null/"" means clear board.
  if (typeof boardId !== "undefined") {
    removeTagsForKey(next, BOARD_TAG_KEY);
    const normalizedBoardId = boardId?.trim() ?? "";
    if (normalizedBoardId.length > 0) {
      pushTagUnique(next, `${BOARD_TAG_KEY}:${normalizedBoardId}`);
    }
  }

  return next;
}

export function boardIdFromTaskTags(tags: string[]): string | null {
  return firstTagValue(tags, BOARD_TAG_KEY);
}

export function kanbanLaneFromTask(tags: string[], availableLanes: string[], fallbackLane: string): string {
  const lane = firstTagValue(tags, KANBAN_TAG_KEY);
  if (!lane) {
    return fallbackLane;
  }
  if (!availableLanes.includes(lane)) {
    return fallbackLane;
  }
  return lane;
}

export function humanizeLane(value: string): string {
  return value
    .split(/[-_]/g)
    .filter((part) => part.length > 0)
    .map((part) => `${part[0]?.toUpperCase() ?? ""}${part.slice(1).toLowerCase()}`)
    .join(" ");
}
