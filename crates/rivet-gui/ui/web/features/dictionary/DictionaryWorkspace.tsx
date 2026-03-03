import { useEffect, useMemo, useState } from "react";
import type { KeyboardEvent } from "react";

import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

import { useDictionaryWorkspaceSlice } from "../../store/slices";

export function DictionaryWorkspace() {
  const {
    runtimeConfig,
    dictionaryLanguages,
    dictionaryLanguage,
    dictionaryQuery,
    dictionaryLoading,
    dictionaryError,
    dictionaryResults,
    dictionaryTotal,
    dictionaryTruncated,
    dictionaryWarnings,
    dictionaryEntry,
    dictionarySelectedId,
    loadDictionaryLanguages,
    setDictionaryLanguage,
    setDictionaryQuery,
    searchDictionaryEntries,
    selectDictionaryHit
  } = useDictionaryWorkspaceSlice();
  const [searchInput, setSearchInput] = useState(dictionaryQuery);
  const [cursorIndex, setCursorIndex] = useState(0);

  useEffect(() => {
    void loadDictionaryLanguages();
  }, [loadDictionaryLanguages]);

  useEffect(() => {
    setSearchInput(dictionaryQuery);
  }, [dictionaryQuery]);

  useEffect(() => {
    if (dictionaryResults.length === 0) {
      setCursorIndex(0);
      return;
    }
    if (cursorIndex >= dictionaryResults.length) {
      setCursorIndex(dictionaryResults.length - 1);
    }
  }, [cursorIndex, dictionaryResults]);

  useEffect(() => {
    if (dictionaryResults.length === 0) {
      return;
    }
    const selectedIndex = dictionaryResults.findIndex((hit) => hit.id !== null && hit.id === dictionarySelectedId);
    if (selectedIndex >= 0) {
      setCursorIndex(selectedIndex);
    }
  }, [dictionaryResults, dictionarySelectedId]);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setDictionaryQuery(searchInput);
      if (searchInput.trim().length === 0) {
        return;
      }
      void searchDictionaryEntries();
    }, 180);
    return () => window.clearTimeout(timeout);
  }, [searchInput, searchDictionaryEntries, setDictionaryQuery]);

  const dictionaryEnabled = runtimeConfig?.dictionary?.enabled ?? true;
  const dbPath = runtimeConfig?.dictionary?.sqlite_path ?? "wiktionary.sqlite";
  const hasResults = dictionaryResults.length > 0;
  const displayLanguage = dictionaryLanguage ?? "__all__";

  const firstWarning = useMemo(() => dictionaryWarnings[0] ?? null, [dictionaryWarnings]);
  const orderedSenses = useMemo(() => {
    if (!dictionaryEntry) {
      return [];
    }
    if (dictionaryEntry.senses.length > 0) {
      return [...dictionaryEntry.senses].sort((left, right) => left.order - right.order);
    }
    return dictionaryEntry.definitions.map((text, index) => ({ order: index + 1, text }));
  }, [dictionaryEntry]);

  const handleSearchKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (dictionaryResults.length === 0) {
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setCursorIndex((previous) => {
        const next = Math.min(dictionaryResults.length - 1, previous + 1);
        const hit = dictionaryResults[next];
        if (hit) {
          void selectDictionaryHit(hit);
        }
        return next;
      });
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setCursorIndex((previous) => {
        const next = Math.max(0, previous - 1);
        const hit = dictionaryResults[next];
        if (hit) {
          void selectDictionaryHit(hit);
        }
        return next;
      });
      return;
    }
    if (event.key === "Enter") {
      const hit = dictionaryResults[cursorIndex];
      if (!hit) {
        return;
      }
      event.preventDefault();
      void selectDictionaryHit(hit);
    }
  };

  if (!dictionaryEnabled) {
    return (
      <div className="h-full p-3">
        <Alert severity="info">Dictionary is disabled in config (`[dictionary].enabled = false`).</Alert>
      </div>
    );
  }

  return (
    <div className="grid h-full min-h-0 grid-cols-[300px_minmax(0,1fr)] gap-3 p-3">
      <Paper className="min-h-0 p-3">
        <Stack spacing={1.25} className="h-full min-h-0">
          <Typography variant="h6">Dictionary</Typography>
          <Typography variant="caption" color="text.secondary">
            db: {dbPath}
          </Typography>
          <TextField
            select
            label="Language"
            size="small"
            value={displayLanguage}
            onChange={(event) => {
              const next = event.target.value === "__all__" ? null : event.target.value;
              setDictionaryLanguage(next);
              if (searchInput.trim().length > 0) {
                void searchDictionaryEntries();
              }
            }}
          >
            <MenuItem value="__all__">All Languages</MenuItem>
            {dictionaryLanguages.map((language) => (
              <MenuItem key={language} value={language}>
                {language}
              </MenuItem>
            ))}
          </TextField>
          <TextField
            label="Search word"
            size="small"
            value={searchInput}
            onChange={(event) => setSearchInput(event.target.value)}
            onKeyDown={handleSearchKeyDown}
            placeholder="type to search..."
          />
          <Stack direction="row" spacing={1}>
            <Button
              variant="contained"
              size="small"
              disabled={dictionaryLoading || searchInput.trim().length === 0}
              onClick={() => {
                setDictionaryQuery(searchInput);
                void searchDictionaryEntries();
              }}
            >
              Search
            </Button>
            <Button
              variant="outlined"
              size="small"
              onClick={() => {
                setSearchInput("");
                setDictionaryQuery("");
              }}
            >
              Clear
            </Button>
          </Stack>

          <Typography variant="caption" color="text.secondary">
            matches: {hasResults ? dictionaryResults.length : 0}/{dictionaryTotal}
            {dictionaryTruncated ? " (truncated)" : ""}
          </Typography>

          {firstWarning ? <Alert severity="warning">{firstWarning}</Alert> : null}
          {dictionaryError ? <Alert severity="error">{dictionaryError}</Alert> : null}

          <Stack spacing={0.75} className="min-h-0 flex-1 overflow-y-auto pr-1">
            {!hasResults && searchInput.trim().length > 0 ? (
              <Alert severity="info">No dictionary results for this query.</Alert>
            ) : null}
            {dictionaryResults.map((hit, index) => {
              const active = hit.id !== null && dictionarySelectedId === hit.id;
              const key = `${hit.id ?? "none"}:${hit.word}:${hit.language ?? ""}`;
              return (
                <Button
                  key={key}
                  variant={active ? "contained" : "outlined"}
                  onClick={() => {
                    void selectDictionaryHit(hit);
                  }}
                  className="!justify-start"
                  color={index === cursorIndex ? "secondary" : "primary"}
                >
                  <span className="truncate">{hit.word}</span>
                </Button>
              );
            })}
          </Stack>
        </Stack>
      </Paper>

      <Paper className="min-h-0 overflow-y-auto p-4">
        {!dictionaryEntry ? (
          <Alert severity="info">Search and pick a word to view its full entry.</Alert>
        ) : (
          <Stack spacing={1.5}>
            <Typography variant="h5">{dictionaryEntry.word}</Typography>
            <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
              {dictionaryEntry.language ? (
                <Typography variant="caption" className="rounded-md border border-current/15 px-1.5 py-0.5">
                  lang:{dictionaryEntry.language}
                </Typography>
              ) : null}
              {dictionaryEntry.part_of_speech ? (
                <Typography variant="caption" className="rounded-md border border-current/15 px-1.5 py-0.5">
                  pos:{dictionaryEntry.part_of_speech}
                </Typography>
              ) : null}
              {dictionaryEntry.source_table ? (
                <Typography variant="caption" className="rounded-md border border-current/15 px-1.5 py-0.5">
                  table:{dictionaryEntry.source_table}
                </Typography>
              ) : null}
            </Stack>
            {dictionaryEntry.pronunciation ? (
              <div>
                <Typography variant="subtitle2">Pronunciation</Typography>
                <Typography variant="body2">{dictionaryEntry.pronunciation}</Typography>
              </div>
            ) : null}
            {dictionaryEntry.etymology ? (
              <div>
                <Typography variant="subtitle2">Etymology</Typography>
                <Typography variant="body2">{dictionaryEntry.etymology}</Typography>
              </div>
            ) : null}
            {orderedSenses.length > 0 ? (
              <div>
                <Typography variant="subtitle2">Definitions</Typography>
                <ol className="m-0 list-decimal pl-5">
                  {orderedSenses.map((sense) => (
                    <li key={`${sense.order}:${sense.text}`} className="mb-1">
                      <Typography variant="body2">{sense.text}</Typography>
                    </li>
                  ))}
                </ol>
              </div>
            ) : null}
            {dictionaryEntry.examples.length > 0 ? (
              <div>
                <Typography variant="subtitle2">Examples</Typography>
                <Stack spacing={0.75}>
                  {dictionaryEntry.examples.map((example) => (
                    <Typography key={example} variant="body2" color="text.secondary">
                      {example}
                    </Typography>
                  ))}
                </Stack>
              </div>
            ) : null}
            {dictionaryEntry.notes.length > 0 ? (
              <div>
                <Typography variant="subtitle2">Notes</Typography>
                <Stack spacing={0.75}>
                  {dictionaryEntry.notes.map((note) => (
                    <Typography key={note} variant="body2" color="text.secondary">
                      {note}
                    </Typography>
                  ))}
                </Stack>
              </div>
            ) : null}
          </Stack>
        )}
      </Paper>
    </div>
  );
}
