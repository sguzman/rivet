import type { ExternalCalendarSource } from "../types/core";
import type { KanbanBoardDef } from "../types/ui";
import { logger } from "./logger";
import { normalizeMarkerColor } from "./tags";

export const THEME_STORAGE_KEY = "rivet.theme";
export const WORKSPACE_TAB_STORAGE_KEY = "rivet.workspace_tab";
export const CALENDAR_VIEW_STORAGE_KEY = "rivet.calendar.view";
export const KANBAN_BOARDS_STORAGE_KEY = "rivet.kanban.boards";
export const KANBAN_ACTIVE_BOARD_STORAGE_KEY = "rivet.kanban.active_board";
export const EXTERNAL_CALENDARS_STORAGE_KEY = "rivet.external_calendars";
export const KANBAN_COMPACT_CARDS_STORAGE_KEY = "rivet.kanban.compact_cards";

function readStorageItem(key: string): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  try {
    return window.localStorage.getItem(key);
  } catch (error) {
    logger.warn("storage.read", `${key}: ${String(error)}`);
    return null;
  }
}

function writeStorageItem(key: string, value: string): void {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.setItem(key, value);
  } catch (error) {
    logger.warn("storage.write", `${key}: ${String(error)}`);
  }
}

function removeStorageItem(key: string): void {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.removeItem(key);
  } catch (error) {
    logger.warn("storage.remove", `${key}: ${String(error)}`);
  }
}

function parseJson<T>(raw: string | null): T | null {
  if (!raw) {
    return null;
  }
  try {
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

function defaultBoardColor(): string {
  return "hsl(212 74% 54%)";
}

function boardColorCandidate(seed: number): string {
  const hue = (seed * 47) % 360;
  return `hsl(${hue} 74% 54%)`;
}

export function nextBoardColor(boards: KanbanBoardDef[]): string {
  const used = new Set(boards.map((board) => board.color.trim().toLowerCase()));
  for (let offset = 0; offset < 512; offset += 1) {
    const candidate = boardColorCandidate(boards.length + offset);
    if (!used.has(candidate.toLowerCase())) {
      return candidate;
    }
  }
  return defaultBoardColor();
}

export function assignUniqueBoardColors(boards: KanbanBoardDef[]): KanbanBoardDef[] {
  const used = new Set<string>();
  return boards.map((board, index) => {
    let color = board.color.trim();
    if (!color) {
      color = boardColorCandidate(index);
    }
    let key = color.toLowerCase();
    if (used.has(key)) {
      for (let offset = 0; offset < 512; offset += 1) {
        const candidate = boardColorCandidate(index + offset);
        const candidateKey = candidate.toLowerCase();
        if (!used.has(candidateKey)) {
          color = candidate;
          key = candidateKey;
          break;
        }
      }
    }
    used.add(key);
    return {
      ...board,
      color
    };
  });
}

export function makeUniqueBoardName(boards: KanbanBoardDef[], requested: string, exceptBoardId = ""): string {
  const base = requested.trim();
  if (!base) {
    return "Board";
  }
  let candidate = base;
  let suffix = 2;
  while (boards.some((board) => board.id !== exceptBoardId && board.name.toLowerCase() === candidate.toLowerCase())) {
    candidate = `${base} ${suffix}`;
    suffix += 1;
  }
  return candidate;
}

function externalCalendarColorCandidate(seed: number): string {
  const hue = ((seed * 53) + 12) % 360;
  return hslToHexColor(hue, 0.72, 0.52);
}

function defaultExternalCalendarColor(): string {
  return externalCalendarColorCandidate(0);
}

function hslToHexColor(hue: number, saturation: number, lightness: number): string {
  const c = (1 - Math.abs((2 * lightness) - 1)) * saturation;
  const hPrime = hue / 60;
  const x = c * (1 - Math.abs((hPrime % 2) - 1));
  let r1 = 0;
  let g1 = 0;
  let b1 = 0;

  if (hPrime >= 0 && hPrime < 1) {
    r1 = c;
    g1 = x;
  } else if (hPrime >= 1 && hPrime < 2) {
    r1 = x;
    g1 = c;
  } else if (hPrime >= 2 && hPrime < 3) {
    g1 = c;
    b1 = x;
  } else if (hPrime >= 3 && hPrime < 4) {
    g1 = x;
    b1 = c;
  } else if (hPrime >= 4 && hPrime < 5) {
    r1 = x;
    b1 = c;
  } else {
    r1 = c;
    b1 = x;
  }

  const m = lightness - c / 2;
  const toHex = (value: number) => Math.round((value + m) * 255).toString(16).padStart(2, "0");
  return `#${toHex(r1)}${toHex(g1)}${toHex(b1)}`;
}

export function nextExternalCalendarColor(sources: ExternalCalendarSource[]): string {
  const used = new Set(
    sources
      .map((source) => normalizeMarkerColor(source.color))
      .map((color) => color.trim().toLowerCase())
  );
  for (let offset = 0; offset < 512; offset += 1) {
    const candidate = externalCalendarColorCandidate(sources.length + offset);
    if (!used.has(candidate.toLowerCase())) {
      return candidate;
    }
  }
  return defaultExternalCalendarColor();
}

export function assignUniqueExternalCalendarColors(sources: ExternalCalendarSource[]): ExternalCalendarSource[] {
  const used = new Set<string>();
  return sources.map((source, index) => {
    let color = source.color.trim();
    if (!color) {
      color = externalCalendarColorCandidate(index);
    }
    let key = normalizeMarkerColor(color).trim().toLowerCase();
    if (used.has(key)) {
      for (let offset = 0; offset < 512; offset += 1) {
        const candidate = externalCalendarColorCandidate(index + offset);
        const candidateKey = candidate.toLowerCase();
        if (!used.has(candidateKey)) {
          color = candidate;
          key = candidateKey;
          break;
        }
      }
    }
    used.add(key);
    return {
      ...source,
      color
    };
  });
}

export function loadKanbanBoards(): KanbanBoardDef[] {
  const parsed = parseJson<KanbanBoardDef[]>(readStorageItem(KANBAN_BOARDS_STORAGE_KEY));
  if (parsed && Array.isArray(parsed)) {
    const filtered = parsed.filter((board) => board.id.trim().length > 0 && board.name.trim().length > 0);
    if (filtered.length > 0) {
      return assignUniqueBoardColors(filtered);
    }
  }
  return [
    {
      id: crypto.randomUUID(),
      name: "Main",
      color: defaultBoardColor()
    }
  ];
}

export function saveKanbanBoards(boards: KanbanBoardDef[]): void {
  writeStorageItem(KANBAN_BOARDS_STORAGE_KEY, JSON.stringify(boards));
}

export function loadActiveKanbanBoardId(boards: KanbanBoardDef[]): string | null {
  const stored = readStorageItem(KANBAN_ACTIVE_BOARD_STORAGE_KEY);
  if (stored && boards.some((board) => board.id === stored)) {
    return stored;
  }
  return boards[0]?.id ?? null;
}

export function saveActiveKanbanBoardId(boardId: string | null): void {
  if (boardId) {
    writeStorageItem(KANBAN_ACTIVE_BOARD_STORAGE_KEY, boardId);
  } else {
    removeStorageItem(KANBAN_ACTIVE_BOARD_STORAGE_KEY);
  }
}

export function loadKanbanCompactCards(): boolean {
  return readStorageItem(KANBAN_COMPACT_CARDS_STORAGE_KEY) === "1";
}

export function saveKanbanCompactCards(enabled: boolean): void {
  writeStorageItem(KANBAN_COMPACT_CARDS_STORAGE_KEY, enabled ? "1" : "0");
}

export function loadExternalCalendars(): ExternalCalendarSource[] {
  const parsed = parseJson<ExternalCalendarSource[]>(readStorageItem(EXTERNAL_CALENDARS_STORAGE_KEY));
  if (!parsed || !Array.isArray(parsed)) {
    return [];
  }
  const filtered = parsed.filter((source) => source.id.trim() && source.name.trim() && source.location.trim());
  const normalized = filtered.map((source) => {
    const importedFromFile = source.location.trim().toLowerCase().startsWith("file://");
    return {
      ...source,
      imported_ics_file: source.imported_ics_file || importedFromFile,
      refresh_minutes: importedFromFile ? 0 : source.refresh_minutes
    };
  });
  return assignUniqueExternalCalendarColors(normalized);
}

export function saveExternalCalendars(sources: ExternalCalendarSource[]): void {
  writeStorageItem(EXTERNAL_CALENDARS_STORAGE_KEY, JSON.stringify(sources));
}

export function newExternalCalendarSource(existing: ExternalCalendarSource[]): ExternalCalendarSource {
  return {
    id: crypto.randomUUID(),
    name: "",
    color: nextExternalCalendarColor(existing),
    location: "",
    refresh_minutes: 30,
    enabled: true,
    imported_ics_file: false,
    read_only: true,
    show_reminders: true,
    offline_support: true
  };
}
