import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";
import maplibregl from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";

import { logger } from "../../lib/logger";
import { useMapWorkspaceSlice } from "../../store/slices";
import { createMartinStyle } from "./style";
import {
  fetchMartinCatalog,
  fetchMartinTileJson,
  normalizeMartinBaseUrl,
  type MartinCatalogSource,
  type MartinTileJson
} from "./martin";

const DEFAULT_CENTER: [number, number] = [-102.2, 28.9];
const DEFAULT_ZOOM = 3.6;
const DEFAULT_MIN_ZOOM = 2;
const DEFAULT_MAX_ZOOM = 13;

type LoadState = "idle" | "loading" | "ready" | "error";

function parseConfiguredCenter(raw: unknown): [number, number] | null {
  if (!Array.isArray(raw) || raw.length !== 2) {
    return null;
  }
  const longitude = typeof raw[0] === "number" ? raw[0] : NaN;
  const latitude = typeof raw[1] === "number" ? raw[1] : NaN;
  if (!Number.isFinite(longitude) || !Number.isFinite(latitude)) {
    return null;
  }
  return [longitude, latitude];
}

function shortError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

interface TileStats {
  requested: number;
  loaded: number;
  errors: number;
}

export function MapWorkspace() {
  const {
    runtimeConfig,
    mapViewportCenter,
    mapViewportZoom,
    mapLastError,
    setMapViewport,
    setMapLastError
  } = useMapWorkspaceSlice();
  const mapConfig = runtimeConfig?.map;
  const mapEnabled = mapConfig?.enabled ?? true;
  const martinBaseUrl = normalizeMartinBaseUrl(mapConfig?.martin_base_url);
  const defaultCenter = parseConfiguredCenter(mapConfig?.default_center) ?? DEFAULT_CENTER;
  const defaultZoom = typeof mapConfig?.default_zoom === "number" ? mapConfig.default_zoom : DEFAULT_ZOOM;
  const minZoom = typeof mapConfig?.min_zoom === "number" ? mapConfig.min_zoom : DEFAULT_MIN_ZOOM;
  const maxZoom = typeof mapConfig?.max_zoom === "number" ? mapConfig.max_zoom : DEFAULT_MAX_ZOOM;
  const maxParallelImageRequests = typeof mapConfig?.max_parallel_image_requests === "number"
    ? mapConfig.max_parallel_image_requests
    : null;
  const cancelPendingTileRequestsWhileZooming = mapConfig?.cancel_pending_tile_requests_while_zooming ?? true;
  const configuredDefaultSource = typeof mapConfig?.default_source === "string" ? mapConfig.default_source.trim() : "";

  const mapHost = useMemo(() => {
    try {
      return new URL(martinBaseUrl).host;
    } catch {
      return martinBaseUrl;
    }
  }, [martinBaseUrl]);

  const mapContainerRef = useRef<HTMLDivElement | null>(null);
  const mapRef = useRef<maplibregl.Map | null>(null);
  const tileStatsRef = useRef<TileStats>({ requested: 0, loaded: 0, errors: 0 });
  const loadStartedAtRef = useRef<number>(0);
  const firstTileLoggedRef = useRef(false);
  const renderTimerRef = useRef<number | null>(null);
  const lastApplyFailedForSourceRef = useRef<string | null>(null);
  const tileStatsSyncTimerRef = useRef<number | null>(null);
  const lastTileErrorRef = useRef<string | null>(null);

  const [sources, setSources] = useState<MartinCatalogSource[]>([]);
  const [selectedSourceId, setSelectedSourceId] = useState<string>("");
  const [selectedTileJson, setSelectedTileJson] = useState<MartinTileJson | null>(null);
  const [status, setStatus] = useState<LoadState>("idle");
  const [errorText, setErrorText] = useState<string | null>(null);
  const [tileStats, setTileStats] = useState<TileStats>({ requested: 0, loaded: 0, errors: 0 });
  const [noTilesVisible, setNoTilesVisible] = useState(false);
  const [viewportText, setViewportText] = useState<string>("-");
  const [refreshToken, setRefreshToken] = useState(0);

  const selectedSource = useMemo(
    () => sources.find((entry) => entry.id === selectedSourceId) ?? null,
    [selectedSourceId, sources]
  );

  const updateViewportText = useCallback(() => {
    const map = mapRef.current;
    if (!map) {
      return;
    }
    const center = map.getCenter();
    setViewportText(`lon=${center.lng.toFixed(4)} lat=${center.lat.toFixed(4)} zoom=${map.getZoom().toFixed(2)}`);
  }, []);

  const resetTileStats = useCallback(() => {
    tileStatsRef.current = { requested: 0, loaded: 0, errors: 0 };
    setTileStats(tileStatsRef.current);
    setNoTilesVisible(false);
    firstTileLoggedRef.current = false;
  }, []);

  const syncTileStats = useCallback(() => {
    setTileStats({ ...tileStatsRef.current });
  }, []);

  const scheduleTileStatsSync = useCallback(() => {
    if (tileStatsSyncTimerRef.current !== null) {
      return;
    }
    tileStatsSyncTimerRef.current = window.setTimeout(() => {
      tileStatsSyncTimerRef.current = null;
      syncTileStats();
    }, 120);
  }, [syncTileStats]);

  const ensureMapInstance = useCallback(() => {
    if (mapRef.current) {
      return mapRef.current;
    }
    const container = mapContainerRef.current;
    if (!container) {
      return null;
    }

    loadStartedAtRef.current = performance.now();
    logger.info(
      "map.init.start",
      `host=${mapHost} default_center=${defaultCenter[0].toFixed(4)},${defaultCenter[1].toFixed(4)} default_zoom=${defaultZoom}`
    );
    const initialCenter = mapViewportCenter ?? defaultCenter;
    const initialZoom = mapViewportZoom ?? defaultZoom;
    if (typeof maxParallelImageRequests === "number" && Number.isFinite(maxParallelImageRequests) && maxParallelImageRequests > 0) {
      maplibregl.setMaxParallelImageRequests(Math.round(maxParallelImageRequests));
      logger.info("map.request.limit", `max_parallel_image_requests=${maplibregl.getMaxParallelImageRequests()}`);
    }
    const map = new maplibregl.Map({
      container,
      style: {
        version: 8,
        name: "rivet-empty",
        sources: {},
        layers: [
          {
            id: "rivet-background",
            type: "background",
            paint: {
              "background-color": "#0c1720"
            }
          }
        ]
      },
      center: initialCenter,
      zoom: initialZoom,
      minZoom,
      maxZoom,
      attributionControl: false,
      cancelPendingTileRequestsWhileZooming
    });

    map.addControl(new maplibregl.NavigationControl({ showCompass: false }), "bottom-right");

    map.on("load", () => {
      logger.info("map.init.loaded", `elapsed_ms=${(performance.now() - loadStartedAtRef.current).toFixed(1)}`);
      updateViewportText();
    });

    map.on("movestart", () => {
      resetTileStats();
    });

    map.on("move", () => {
      if (renderTimerRef.current !== null) {
        window.clearTimeout(renderTimerRef.current);
      }
      renderTimerRef.current = window.setTimeout(() => {
        const center = map.getCenter();
        setMapViewport([center.lng, center.lat], map.getZoom());
        updateViewportText();
      }, 120);
    });

    map.on("sourcedataloading", () => {
      tileStatsRef.current.requested += 1;
      scheduleTileStatsSync();
    });

    map.on("sourcedata", (event) => {
      if (event.dataType !== "source") {
        return;
      }
      tileStatsRef.current.loaded += 1;
      if (!firstTileLoggedRef.current) {
        firstTileLoggedRef.current = true;
        logger.info(
          "map.tile.first_render",
          `elapsed_ms=${(performance.now() - loadStartedAtRef.current).toFixed(1)}`
        );
      }
      scheduleTileStatsSync();
    });

    map.on("error", (event) => {
      tileStatsRef.current.errors += 1;
      scheduleTileStatsSync();
      const detail = event.error instanceof Error ? event.error.message : String(event.error);
      if (lastTileErrorRef.current !== detail) {
        lastTileErrorRef.current = detail;
        setMapLastError(detail);
      }
      logger.warn("map.tile.error", detail);
    });

    map.on("idle", () => {
      const stats = tileStatsRef.current;
      const noTiles = stats.requested > 0 && stats.loaded === 0 && stats.errors > 0;
      setNoTilesVisible(noTiles);
      if (noTiles) {
        setMapLastError("No tiles are available for the current viewport.");
        logger.warn("map.tiles.none_visible", `requested=${stats.requested} errors=${stats.errors}`);
      }
    });

    mapRef.current = map;
    return map;
  }, [cancelPendingTileRequestsWhileZooming, defaultCenter, defaultZoom, mapHost, mapViewportCenter, mapViewportZoom, maxParallelImageRequests, maxZoom, minZoom, resetTileStats, scheduleTileStatsSync, setMapLastError, setMapViewport, updateViewportText]);

  const loadCatalog = useCallback(async () => {
    setStatus("loading");
    setErrorText(null);
    setSelectedTileJson(null);
    try {
      const catalog = await fetchMartinCatalog(martinBaseUrl);
      if (catalog.length === 0) {
        throw new Error("Martin returned zero tile sources from /catalog");
      }
      setSources(catalog);
      const preferred = catalog.find((entry) => entry.id === configuredDefaultSource) ?? catalog[0];
      if (!preferred) {
        throw new Error("No selectable Martin source found");
      }
      setSelectedSourceId(preferred.id);
      setStatus("ready");
    } catch (error) {
      const message = shortError(error);
      logger.error("map.martin.catalog.error", message);
      setMapLastError(message);
      setSources([]);
      setSelectedSourceId("");
      setStatus("error");
      setErrorText(message);
    }
  }, [configuredDefaultSource, martinBaseUrl, setMapLastError]);

  useEffect(() => {
    if (!mapEnabled) {
      return;
    }
    void loadCatalog();
  }, [loadCatalog, mapEnabled, refreshToken]);

  useEffect(() => {
    if (!mapEnabled || !selectedSource) {
      return;
    }
    if (lastApplyFailedForSourceRef.current === selectedSource.id) {
      return;
    }

    let cancelled = false;
    const applySource = async () => {
      setStatus("loading");
      setErrorText(null);
      setMapLastError(null);
      lastTileErrorRef.current = null;
      resetTileStats();
      try {
        const tilejson = await fetchMartinTileJson(selectedSource.tilejson_url);
        if (cancelled) {
          return;
        }
        setSelectedTileJson(tilejson);
        setMapLastError(null);

        const map = ensureMapInstance();
        if (!map) {
          throw new Error("Map container is not available");
        }

        const style = createMartinStyle(selectedSource.id, tilejson);
        logger.info(
          "map.source.apply",
          `source=${selectedSource.id} vector_layers=${tilejson.vector_layers.length} minzoom=${tilejson.minzoom} maxzoom=${tilejson.maxzoom}`
        );
        loadStartedAtRef.current = performance.now();
        map.setStyle(style);
        map.fitBounds(
          tilejson.bounds ?? [-128.5, 14.0, -86.0, 33.8],
          { padding: 28, duration: 0, maxZoom: 7 }
        );
        lastApplyFailedForSourceRef.current = null;
        setStatus("ready");
      } catch (error) {
        if (cancelled) {
          return;
        }
        lastApplyFailedForSourceRef.current = selectedSource.id;
        const message = shortError(error);
        logger.error("map.source.apply.error", `${selectedSource.id}: ${message}`);
        setMapLastError(message);
        setStatus("error");
        setErrorText(message);
      }
    };

    void applySource();
    return () => {
      cancelled = true;
    };
  }, [ensureMapInstance, mapEnabled, resetTileStats, selectedSource, setMapLastError]);

  useEffect(() => {
    return () => {
      if (renderTimerRef.current !== null) {
        window.clearTimeout(renderTimerRef.current);
      }
      if (tileStatsSyncTimerRef.current !== null) {
        window.clearTimeout(tileStatsSyncTimerRef.current);
      }
      mapRef.current?.remove();
      mapRef.current = null;
    };
  }, []);

  if (!mapEnabled) {
    return (
      <div className="h-full p-3">
        <Alert severity="info">Map is disabled in config (`[map].enabled = false`).</Alert>
      </div>
    );
  }

  return (
    <div className="h-full p-3">
      <Paper className="grid h-full min-h-0 grid-rows-[auto_minmax(0,1fr)_auto] gap-2 p-2">
        <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap" useFlexGap>
          <Typography variant="h6">Map</Typography>
          <Typography variant="caption" color="text.secondary">martin: {martinBaseUrl}</Typography>
          <Typography variant="caption" color="text.secondary">state: {status}</Typography>
          <Typography variant="caption" color="text.secondary">viewport: {viewportText}</Typography>
          <div className="ml-auto" />
          <TextField
            select
            size="small"
            label="Source"
            className="min-w-[240px]"
            value={selectedSourceId}
            disabled={sources.length === 0 || status === "loading"}
            onChange={(event) => setSelectedSourceId(event.target.value)}
          >
            {sources.map((source) => (
              <MenuItem key={source.id} value={source.id}>{source.title}</MenuItem>
            ))}
          </TextField>
          <Button size="small" variant="outlined" onClick={() => setRefreshToken((value) => value + 1)}>Reload Sources</Button>
          <Button size="small" variant="outlined" onClick={() => mapRef.current?.zoomIn()}>+</Button>
          <Button size="small" variant="outlined" onClick={() => mapRef.current?.zoomOut()}>-</Button>
          <Button
            size="small"
            variant="outlined"
            onClick={() => {
              const map = mapRef.current;
              if (!map) {
                return;
              }
              map.jumpTo({ center: defaultCenter, zoom: defaultZoom });
            }}
          >
            Reset
          </Button>
        </Stack>

        <div className="relative min-h-0 overflow-hidden rounded-md border border-current/15">
          <div ref={mapContainerRef} className="h-full w-full" />
          {status === "loading" ? (
            <div className="pointer-events-none absolute inset-0 flex items-center justify-center bg-black/30">
              <Paper className="px-3 py-2">
                <Typography variant="body2">Loading Martin tiles...</Typography>
              </Paper>
            </div>
          ) : null}
        </div>

        <Stack spacing={0.75}>
          {errorText ? <Alert severity="error">Map load failed: {errorText}</Alert> : null}
          {!errorText && mapLastError ? <Alert severity="warning">Map warning: {mapLastError}</Alert> : null}
          {noTilesVisible ? <Alert severity="warning">No tiles are available for this zoom/area.</Alert> : null}
          <Typography variant="caption" color="text.secondary">
            source={selectedSource?.id ?? "(none)"}
            {selectedTileJson ? ` tilejson=${selectedTileJson.tilejson} layers=${selectedTileJson.vector_layers.length}` : ""}
            {` tile_requests=${tileStats.requested} tile_loaded=${tileStats.loaded} tile_errors=${tileStats.errors}`}
          </Typography>
        </Stack>
      </Paper>
    </div>
  );
}
