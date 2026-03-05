# Martin Map Tab: Startup and Troubleshooting

## Local Startup

1. Start Martin from the project root:

```bash
docker compose -f tmp/docker-compose.yaml up -d
```

2. Confirm Martin is reachable:

```bash
curl -sS http://127.0.0.1:3002/catalog
```

3. Start Rivet and open the `Map` tab.

## Expected Runtime

- Map tab loads source metadata from `http://127.0.0.1:3002/catalog`.
- A source is selected and TileJSON is loaded from `http://127.0.0.1:3002/<source>`.
- Pan/zoom/reset controls remain responsive.
- If you leave US/Mexico coverage, the UI may show `No tiles are available for this zoom/area`.

## Common Failures

- `Map load failed: HTTP 500 ...`:
  - Martin is up but source discovery failed. Check Martin logs and tile mount path.
- `Map load failed: Failed to fetch`:
  - Martin is down or unreachable from desktop runtime.
- Empty map with no source options:
  - `/catalog` returned no sources. Verify `/win/services/martin/tiles` contains readable `*.mbtiles`/`*.pmtiles`.

## Quick Validation Checklist

1. `curl http://127.0.0.1:3002/catalog` returns JSON with at least one source.
2. Map tab shows `state: ready`.
3. `+`, `-`, and `Reset` buttons change viewport.
4. Switching tabs away/from Map preserves the current viewport.
