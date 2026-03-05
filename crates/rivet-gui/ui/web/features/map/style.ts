import type { LayerSpecification, StyleSpecification } from "maplibre-gl";

import type { MartinTileJson } from "./martin";

function vectorLayersForSource(sourceId: string, tilejson: MartinTileJson): LayerSpecification[] {
  const layers: LayerSpecification[] = [];
  for (const vectorLayer of tilejson.vector_layers) {
    const layerBase = `${sourceId}-${vectorLayer.id}`;
    layers.push({
      id: `${layerBase}-fill`,
      type: "fill",
      source: sourceId,
      "source-layer": vectorLayer.id,
      filter: ["==", ["geometry-type"], "Polygon"],
      paint: {
        "fill-color": "#2f77a5",
        "fill-opacity": 0.2
      }
    });
    layers.push({
      id: `${layerBase}-line`,
      type: "line",
      source: sourceId,
      "source-layer": vectorLayer.id,
      filter: ["==", ["geometry-type"], "LineString"],
      paint: {
        "line-color": "#8ec4e7",
        "line-width": 1.1,
        "line-opacity": 0.88
      }
    });
    layers.push({
      id: `${layerBase}-circle`,
      type: "circle",
      source: sourceId,
      "source-layer": vectorLayer.id,
      filter: ["==", ["geometry-type"], "Point"],
      paint: {
        "circle-color": "#7ad0a8",
        "circle-radius": 2.2,
        "circle-opacity": 0.92
      }
    });
  }
  return layers;
}

export function createMartinStyle(sourceId: string, tilejsonUrl: string, tilejson: MartinTileJson): StyleSpecification {
  const background: LayerSpecification = {
    id: "rivet-map-background",
    type: "background",
    paint: {
      "background-color": "#09141d"
    }
  };

  const style: StyleSpecification = {
    version: 8,
    name: `rivet-${sourceId}`,
    sources: {},
    layers: [background]
  };

  if (tilejson.vector_layers.length > 0) {
    style.sources[sourceId] = {
      type: "vector",
      url: tilejsonUrl,
      minzoom: tilejson.minzoom,
      maxzoom: tilejson.maxzoom
    };
    style.layers = [background, ...vectorLayersForSource(sourceId, tilejson)];
    return style;
  }

  style.sources[sourceId] = {
    type: "raster",
    tiles: tilejson.tiles,
    tileSize: 256,
    minzoom: tilejson.minzoom,
    maxzoom: tilejson.maxzoom
  };
  style.layers.push({
    id: `${sourceId}-raster`,
    type: "raster",
    source: sourceId,
    paint: {
      "raster-opacity": 1
    }
  });
  return style;
}
