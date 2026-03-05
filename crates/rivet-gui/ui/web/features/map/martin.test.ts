import { describe, expect, it } from "vitest";

import { normalizeMartinBaseUrl, parseMartinCatalog, parseMartinTileJson } from "./martin";

describe("map martin helpers", () => {
  it("normalizes base url with fallback", () => {
    expect(normalizeMartinBaseUrl(" http://127.0.0.1:3002/ ")).toBe("http://127.0.0.1:3002");
    expect(normalizeMartinBaseUrl("bad:url")).toBe("http://127.0.0.1:3002");
  });

  it("parses catalog source entries", () => {
    const payload = {
      fonts: {
        path: "/fonts/{fontstack}/{range}.pbf"
      },
      sprites: {
        path: "/sprite"
      },
      styles: {
        path: "/styles"
      },
      tiles: {
        path: "/tiles"
      },
      roads_us: {
        name: "US Roads",
        description: "USA road network",
        tilejson: "./roads_us"
      },
      landuse_mx: {
        name: "MX Landuse"
      }
    };
    const parsed = parseMartinCatalog(payload, "http://127.0.0.1:3002");

    expect(parsed).toHaveLength(2);
    expect(parsed[0]).toMatchObject({
      id: "landuse_mx",
      title: "MX Landuse",
      tilejson_url: "http://127.0.0.1:3002/landuse_mx"
    });
    expect(parsed[1]).toMatchObject({
      id: "roads_us",
      title: "US Roads",
      tilejson_url: "http://127.0.0.1:3002/roads_us"
    });
  });

  it("parses nested tiles catalog shape", () => {
    const payload = {
      tiles: {
        mexico_z14: {
          name: "Mexico z14",
          description: "mexico tiles"
        },
        us_z14: {
          name: "US z14",
          description: "us tiles"
        }
      },
      fonts: {},
      sprites: {},
      styles: {}
    };

    const parsed = parseMartinCatalog(payload, "http://127.0.0.1:3002");
    expect(parsed).toHaveLength(2);
    expect(parsed.map((entry) => entry.id).sort()).toEqual(["mexico_z14", "us_z14"]);
    expect(parsed[0]?.tilejson_url).toMatch(/^http:\/\/127\.0\.0\.1:3002\//);
  });

  it("parses TileJSON and vector layers", () => {
    const payload = {
      tilejson: "3.0.0",
      name: "roads_us",
      tiles: ["http://127.0.0.1:3002/roads_us/{z}/{x}/{y}.pbf"],
      minzoom: 0,
      maxzoom: 14,
      bounds: [-123.0, 16.0, -86.5, 33.9],
      vector_layers: [
        { id: "road", description: "roads" },
        { id: "water" }
      ]
    };

    const parsed = parseMartinTileJson(payload);
    expect(parsed.tiles).toHaveLength(1);
    expect(parsed.vector_layers.map((entry) => entry.id)).toEqual(["road", "water"]);
    expect(parsed.bounds).toEqual([-123.0, 16.0, -86.5, 33.9]);
  });

  it("rejects invalid TileJSON payloads", () => {
    expect(() => parseMartinTileJson({ tilejson: "3.0.0" })).toThrow("TileJSON missing tiles[]");
  });
});
