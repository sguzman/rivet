// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { MapWorkspace } from "./MapWorkspace";

const mapMock = vi.hoisted(() => {
  const state = {
    maxParallelImageRequests: 16,
    zoomIn: vi.fn(),
    zoomOut: vi.fn(),
    jumpTo: vi.fn(),
    setStyle: vi.fn(),
    fitBounds: vi.fn(),
    remove: vi.fn(),
    handlers: new Map<string, Array<() => void>>()
  };
  class MockMap {
    on(event: string, handler: () => void) {
      const existing = state.handlers.get(event) ?? [];
      existing.push(handler);
      state.handlers.set(event, existing);
      return this;
    }

    addControl() {
      return this;
    }

    setStyle(...args: unknown[]) {
      state.setStyle(...args);
      return this;
    }

    fitBounds(...args: unknown[]) {
      state.fitBounds(...args);
      return this;
    }

    zoomIn() {
      state.zoomIn();
      return this;
    }

    zoomOut() {
      state.zoomOut();
      return this;
    }

    jumpTo(...args: unknown[]) {
      state.jumpTo(...args);
      return this;
    }

    getCenter() {
      return { lng: -99.1332, lat: 19.4326 };
    }

    getZoom() {
      return 6;
    }

    remove() {
      state.remove();
    }
  }

  class MockNavigationControl {}

  return {
    state,
    module: {
      default: {
        Map: MockMap,
        NavigationControl: MockNavigationControl,
        getMaxParallelImageRequests: () => state.maxParallelImageRequests,
        setMaxParallelImageRequests: (value: number) => {
          state.maxParallelImageRequests = value;
        }
      }
    }
  };
});

vi.mock("maplibre-gl", () => mapMock.module);

const mockSlice = vi.fn();

vi.mock("../../store/slices", () => ({
  useMapWorkspaceSlice: () => mockSlice()
}));

const baseSlice = {
  runtimeConfig: {
    map: {
      enabled: true,
      martin_base_url: "http://127.0.0.1:3002",
      default_center: [-102.2, 28.9],
      default_zoom: 3.6,
      min_zoom: 2,
      max_zoom: 13,
      max_parallel_image_requests: 12,
      cancel_pending_tile_requests_while_zooming: true
    }
  },
  mapViewportCenter: null,
  mapViewportZoom: null,
  mapLastError: null,
  setMapViewport: vi.fn(),
  setMapLastError: vi.fn()
};

describe("MapWorkspace", () => {
  let stableSlice: typeof baseSlice;

  beforeEach(() => {
    cleanup();
    vi.restoreAllMocks();
    vi.clearAllMocks();
    mapMock.state.handlers.clear();
    stableSlice = {
      ...baseSlice,
      setMapViewport: vi.fn(),
      setMapLastError: vi.fn()
    };
    mockSlice.mockImplementation(() => stableSlice);
  });

  it("shows disabled state when map feature is off", () => {
    mockSlice.mockReturnValue({
      ...stableSlice,
      runtimeConfig: {
        map: {
          ...stableSlice.runtimeConfig.map,
          enabled: false
        }
      }
    });

    render(<MapWorkspace />);

    expect(screen.getByText("Map is disabled in config (`[map].enabled = false`).")).toBeTruthy();
  });

  it("renders map controls and applies zoom/reset interactions", async () => {
    vi.spyOn(globalThis, "fetch").mockImplementation((input: RequestInfo | URL) => {
      const url = typeof input === "string" ? input : input.toString();
      if (url.endsWith("/catalog")) {
        return Promise.resolve(new Response(JSON.stringify({ mexico: { name: "Mexico" } }), { status: 200 }));
      }
      return Promise.resolve(new Response(JSON.stringify({
        tilejson: "3.0.0",
        tiles: ["http://127.0.0.1:3002/mexico/{z}/{x}/{y}.pbf"],
        minzoom: 0,
        maxzoom: 14,
        bounds: [-118, 14, -86, 33],
        vector_layers: [{ id: "roads" }]
      }), { status: 200 }));
    });

    render(<MapWorkspace />);

    await waitFor(() => expect(screen.getByText("state: ready")).toBeTruthy());

    fireEvent.click(screen.getByRole("button", { name: "+" }));
    fireEvent.click(screen.getByRole("button", { name: "-" }));
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));

    expect(mapMock.state.zoomIn).toHaveBeenCalledTimes(1);
    expect(mapMock.state.zoomOut).toHaveBeenCalledTimes(1);
    expect(mapMock.state.jumpTo).toHaveBeenCalledTimes(1);
  });

  it("shows error alert when Martin catalog fetch fails", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce(new Response("boom", { status: 500, statusText: "Server Error" }));

    render(<MapWorkspace />);

    await waitFor(() => {
      expect(screen.getByText(/Map load failed:/)).toBeTruthy();
    });
  });
});
