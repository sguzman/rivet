import { logger } from "../../lib/logger";

export interface MartinCatalogSource {
  id: string;
  title: string;
  description: string | null;
  tilejson_url: string;
}

export interface MartinVectorLayer {
  id: string;
  description?: string;
}

export interface MartinTileJson {
  tilejson: string;
  name: string | null;
  description: string | null;
  tiles: string[];
  minzoom: number;
  maxzoom: number;
  bounds: [number, number, number, number] | null;
  vector_layers: MartinVectorLayer[];
}

const DEFAULT_FETCH_TIMEOUT_MS = 8_000;
const RESERVED_CATALOG_IDS = new Set(["fonts", "font", "sprites", "sprite", "catalog", "health"]);

function isObject(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function asNonEmptyString(value: unknown): string | null {
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function numberOrFallback(value: unknown, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function parseBounds(value: unknown): [number, number, number, number] | null {
  if (!Array.isArray(value) || value.length !== 4) {
    return null;
  }
  const parsed = value.map((item) => (typeof item === "number" && Number.isFinite(item) ? item : NaN));
  if (parsed.some((item) => Number.isNaN(item))) {
    return null;
  }
  return [parsed[0], parsed[1], parsed[2], parsed[3]];
}

function toAbsoluteUrl(baseUrl: string, value: string): string {
  try {
    return new URL(value, `${baseUrl}/`).toString();
  } catch {
    return `${baseUrl}/${value.replace(/^\/+/, "")}`;
  }
}

async function fetchJsonWithTimeout<T>(url: string, timeoutMs = DEFAULT_FETCH_TIMEOUT_MS): Promise<T> {
  const controller = new AbortController();
  const timeout = window.setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(url, {
      method: "GET",
      headers: {
        Accept: "application/json"
      },
      signal: controller.signal
    });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status} ${response.statusText}`);
    }
    const payload = await response.json() as T;
    return payload;
  } finally {
    window.clearTimeout(timeout);
  }
}

export function normalizeMartinBaseUrl(raw: string | undefined | null): string {
  const fallback = "http://127.0.0.1:3002";
  const candidate = asNonEmptyString(raw) ?? fallback;
  try {
    const url = new URL(candidate);
    if (url.protocol !== "http:" && url.protocol !== "https:") {
      return fallback;
    }
    const normalizedPath = url.pathname.replace(/\/+$/, "");
    url.pathname = normalizedPath;
    return url.toString().replace(/\/+$/, "");
  } catch {
    return fallback;
  }
}

export function parseMartinCatalog(payload: unknown, baseUrl: string): MartinCatalogSource[] {
  const root = isObject(payload)
    ? (isObject(payload.sources) ? payload.sources : payload)
    : {};

  const sources: MartinCatalogSource[] = [];
  for (const [key, rawValue] of Object.entries(root)) {
    if (key === "sources") {
      continue;
    }
    const id = key.trim();
    if (!id || RESERVED_CATALOG_IDS.has(id.toLowerCase())) {
      continue;
    }

    const value = isObject(rawValue) ? rawValue : {};
    const title = asNonEmptyString(value.name) ?? id;
    const description = asNonEmptyString(value.description);
    const explicitTileJsonUrl = asNonEmptyString(value.tilejson)
      ?? asNonEmptyString(value.tilejson_url)
      ?? asNonEmptyString(value.url);
    const tilejson_url = explicitTileJsonUrl
      ? toAbsoluteUrl(baseUrl, explicitTileJsonUrl)
      : toAbsoluteUrl(baseUrl, encodeURIComponent(id));

    sources.push({
      id,
      title,
      description,
      tilejson_url
    });
  }

  return sources.sort((left, right) => left.title.localeCompare(right.title));
}

export function parseMartinTileJson(payload: unknown): MartinTileJson {
  if (!isObject(payload)) {
    throw new Error("invalid TileJSON payload");
  }

  const tiles = Array.isArray(payload.tiles)
    ? payload.tiles.filter((item): item is string => typeof item === "string" && item.trim().length > 0)
    : [];
  if (tiles.length === 0) {
    throw new Error("TileJSON missing tiles[]");
  }

  const vector_layers = Array.isArray(payload.vector_layers)
    ? payload.vector_layers
      .filter((item): item is Record<string, unknown> => isObject(item))
      .map((item) => ({
        id: asNonEmptyString(item.id) ?? "",
        description: asNonEmptyString(item.description) ?? undefined
      }))
      .filter((item) => item.id.length > 0)
    : [];

  return {
    tilejson: asNonEmptyString(payload.tilejson) ?? "3.0.0",
    name: asNonEmptyString(payload.name),
    description: asNonEmptyString(payload.description),
    tiles,
    minzoom: Math.max(0, numberOrFallback(payload.minzoom, 0)),
    maxzoom: Math.min(24, Math.max(0, numberOrFallback(payload.maxzoom, 14))),
    bounds: parseBounds(payload.bounds),
    vector_layers
  };
}

export async function fetchMartinCatalog(baseUrl: string): Promise<MartinCatalogSource[]> {
  const catalogUrl = toAbsoluteUrl(baseUrl, "catalog");
  logger.info("map.martin.catalog.start", catalogUrl);
  const started = performance.now();
  const payload = await fetchJsonWithTimeout<unknown>(catalogUrl);
  const sources = parseMartinCatalog(payload, baseUrl);
  logger.info(
    "map.martin.catalog.done",
    `count=${sources.length} elapsed_ms=${(performance.now() - started).toFixed(1)}`
  );
  return sources;
}

export async function fetchMartinTileJson(tilejsonUrl: string): Promise<MartinTileJson> {
  logger.debug("map.martin.tilejson.start", tilejsonUrl);
  const started = performance.now();
  const payload = await fetchJsonWithTimeout<unknown>(tilejsonUrl);
  const parsed = parseMartinTileJson(payload);
  logger.debug(
    "map.martin.tilejson.done",
    `layers=${parsed.vector_layers.length} elapsed_ms=${(performance.now() - started).toFixed(1)}`
  );
  return parsed;
}
