import { useEffect, useMemo, useState } from "react";
import type { KeyboardEvent } from "react";

import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import IconButton from "@mui/material/IconButton";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import StarIcon from "@mui/icons-material/Star";
import StarBorderIcon from "@mui/icons-material/StarBorder";

import { useDictionaryWorkspaceSlice } from "../../store/slices";

const DICTIONARY_FAVORITES_KEY = "rivet.dictionary.favorites";
const DICTIONARY_HISTORY_KEY = "rivet.dictionary.history";

type DictionarySavedEntry = {
  id: number | null;
  word: string;
  language: string | null;
};

function readStoredEntries(key: string): DictionarySavedEntry[] {
  if (typeof window === "undefined") {
    return [];
  }
  try {
    const raw = window.localStorage.getItem(key);
    if (!raw) {
      return [];
    }
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed
      .filter((item): item is DictionarySavedEntry => {
        if (!item || typeof item !== "object" || Array.isArray(item)) {
          return false;
        }
        const word = (item as { word?: unknown }).word;
        const id = (item as { id?: unknown }).id;
        const language = (item as { language?: unknown }).language;
        const idOk = id === null || typeof id === "number";
        const languageOk = language === null || typeof language === "string";
        return typeof word === "string" && word.trim().length > 0 && idOk && languageOk;
      })
      .slice(0, 100);
  } catch {
    return [];
  }
}

function writeStoredEntries(key: string, entries: DictionarySavedEntry[]): void {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.setItem(key, JSON.stringify(entries.slice(0, 100)));
  } catch {
    // no-op
  }
}

function savedKey(entry: DictionarySavedEntry): string {
  return `${entry.word.toLowerCase()}::${entry.language ?? ""}`;
}

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
  const [favorites, setFavorites] = useState<DictionarySavedEntry[]>(() => readStoredEntries(DICTIONARY_FAVORITES_KEY));
  const [history, setHistory] = useState<DictionarySavedEntry[]>(() => readStoredEntries(DICTIONARY_HISTORY_KEY));

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
    if (!dictionaryEntry) {
      return;
    }
    const entry: DictionarySavedEntry = {
      id: dictionaryEntry.id,
      word: dictionaryEntry.word,
      language: dictionaryEntry.language
    };
    setHistory((previous) => {
      const key = savedKey(entry);
      const next = [entry, ...previous.filter((item) => savedKey(item) !== key)].slice(0, 50);
      writeStoredEntries(DICTIONARY_HISTORY_KEY, next);
      return next;
    });
  }, [dictionaryEntry]);

  const dictionaryEnabled = runtimeConfig?.dictionary?.enabled ?? true;
  const postgres = runtimeConfig?.dictionary?.postgres;
  const backendTarget = postgres
    ? `${postgres.host ?? "127.0.0.1"}:${postgres.port ?? 5432}/${postgres.database ?? "data"}.${postgres.schema ?? "dictionary"}`
    : "postgres://127.0.0.1:5432/data.dictionary";
  const searchMode = String(runtimeConfig?.dictionary?.search_mode ?? "prefix").trim().toLowerCase();
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
  const relationBuckets = useMemo(() => {
    const synonyms: string[] = [];
    const antonyms: string[] = [];
    const translations: string[] = [];
    const domains: string[] = [];
    const syllabification: string[] = [];
    const audio: string[] = [];
    const others: string[] = [];
    const metadata = dictionaryEntry?.metadata ?? [];
    for (const item of metadata) {
      const relationType = item.relation_type.toLowerCase();
      const target = item.target;
      if (relationType.includes("synonym")) {
        synonyms.push(target);
      } else if (relationType.includes("antonym")) {
        antonyms.push(target);
      } else if (relationType.includes("translation")) {
        translations.push(target);
      } else if (relationType.includes("domain") || relationType.includes("register")) {
        domains.push(target);
      } else if (relationType.includes("syllab")) {
        syllabification.push(target);
      } else if (relationType.includes("audio") || relationType.includes("sound") || relationType.includes("pronunciation_url")) {
        audio.push(target);
      } else {
        others.push(`${item.relation_type}: ${target}`);
      }
    }
    return { synonyms, antonyms, translations, domains, syllabification, audio, others };
  }, [dictionaryEntry]);
  const isFavorite = useMemo(() => {
    if (!dictionaryEntry) {
      return false;
    }
    const current: DictionarySavedEntry = {
      id: dictionaryEntry.id,
      word: dictionaryEntry.word,
      language: dictionaryEntry.language
    };
    const key = savedKey(current);
    return favorites.some((entry) => savedKey(entry) === key);
  }, [dictionaryEntry, favorites]);

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
  const openSavedEntry = (entry: DictionarySavedEntry) => {
    void selectDictionaryHit({
      id: entry.id,
      word: entry.word,
      language: entry.language,
      part_of_speech: null,
      pronunciation: null,
      summary: null,
      source_table: "saved",
      matched_by_prefix: false
    });
  };
  const toggleFavorite = () => {
    if (!dictionaryEntry) {
      return;
    }
    const current: DictionarySavedEntry = {
      id: dictionaryEntry.id,
      word: dictionaryEntry.word,
      language: dictionaryEntry.language
    };
    const key = savedKey(current);
    setFavorites((previous) => {
      const exists = previous.some((item) => savedKey(item) === key);
      const next = exists
        ? previous.filter((item) => savedKey(item) !== key)
        : [current, ...previous].slice(0, 50);
      writeStoredEntries(DICTIONARY_FAVORITES_KEY, next);
      return next;
    });
  };
  const copyText = async (text: string) => {
    if (typeof navigator === "undefined" || !navigator.clipboard) {
      return;
    }
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      // no-op
    }
  };
  const openTasksSplit = () => {
    window.dispatchEvent(new Event("rivet:dictionary-open-tasks-split"));
  };
  const closeTasksSplit = () => {
    window.dispatchEvent(new Event("rivet:dictionary-close-tasks-split"));
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
            backend: {backendTarget}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            mode: {searchMode}
          </Typography>
          <TextField
            select
            label="Language"
            size="small"
            value={displayLanguage}
            onChange={(event) => {
              const next = event.target.value === "__all__" ? null : event.target.value;
              setDictionaryLanguage(next);
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
            <Button variant="outlined" size="small" onClick={openTasksSplit}>
              Split with Tasks
            </Button>
            <Button variant="text" size="small" onClick={closeTasksSplit}>
              Exit Split
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
          {favorites.length > 0 ? (
            <>
              <Typography variant="subtitle2">Favorites</Typography>
              <Stack spacing={0.5} className="max-h-[120px] overflow-y-auto pr-1">
                {favorites.map((entry) => (
                  <Button key={`fav:${savedKey(entry)}`} size="small" variant="text" className="!justify-start" onClick={() => openSavedEntry(entry)}>
                    {entry.word}
                  </Button>
                ))}
              </Stack>
            </>
          ) : null}
          {history.length > 0 ? (
            <>
              <Typography variant="subtitle2">Recent</Typography>
              <Stack spacing={0.5} className="max-h-[120px] overflow-y-auto pr-1">
                {history.map((entry) => (
                  <Button key={`hist:${savedKey(entry)}`} size="small" variant="text" className="!justify-start" onClick={() => openSavedEntry(entry)}>
                    {entry.word}
                  </Button>
                ))}
              </Stack>
            </>
          ) : null}
        </Stack>
      </Paper>

      <Paper className="min-h-0 overflow-y-auto p-4">
        {!dictionaryEntry ? (
          <Alert severity="info">Search and pick a word to view its full entry.</Alert>
        ) : (
          <Stack spacing={1.5}>
            <Typography variant="h5">{dictionaryEntry.word}</Typography>
            <Stack direction="row" spacing={0.5}>
              <IconButton size="small" onClick={toggleFavorite} aria-label="Toggle favorite">
                {isFavorite ? <StarIcon fontSize="small" /> : <StarBorderIcon fontSize="small" />}
              </IconButton>
              <IconButton
                size="small"
                onClick={() => {
                  const primary = orderedSenses[0]?.text ?? dictionaryEntry.word;
                  void copyText(primary);
                }}
                aria-label="Copy definition"
              >
                <ContentCopyIcon fontSize="small" />
              </IconButton>
            </Stack>
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
                <Stack direction="row" spacing={1} alignItems="center">
                  <Typography variant="subtitle2">Pronunciation (IPA)</Typography>
                  <IconButton size="small" onClick={() => void copyText(dictionaryEntry.pronunciation ?? "")} aria-label="Copy pronunciation">
                    <ContentCopyIcon fontSize="small" />
                  </IconButton>
                </Stack>
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
            {relationBuckets.synonyms.length > 0 ? (
              <div>
                <Typography variant="subtitle2">Synonyms</Typography>
                <Typography variant="body2" color="text.secondary">{relationBuckets.synonyms.join(", ")}</Typography>
              </div>
            ) : null}
            {relationBuckets.antonyms.length > 0 ? (
              <div>
                <Typography variant="subtitle2">Antonyms</Typography>
                <Typography variant="body2" color="text.secondary">{relationBuckets.antonyms.join(", ")}</Typography>
              </div>
            ) : null}
            {relationBuckets.translations.length > 0 ? (
              <div>
                <Typography variant="subtitle2">Translations</Typography>
                <Typography variant="body2" color="text.secondary">{relationBuckets.translations.join(", ")}</Typography>
              </div>
            ) : null}
            {relationBuckets.domains.length > 0 ? (
              <div>
                <Typography variant="subtitle2">Domain/Register</Typography>
                <Typography variant="body2" color="text.secondary">{relationBuckets.domains.join(", ")}</Typography>
              </div>
            ) : null}
            {relationBuckets.syllabification.length > 0 ? (
              <div>
                <Typography variant="subtitle2">Syllabification</Typography>
                <Typography variant="body2" color="text.secondary">{relationBuckets.syllabification.join(" | ")}</Typography>
              </div>
            ) : null}
            {relationBuckets.audio.length > 0 ? (
              <div>
                <Typography variant="subtitle2">Audio</Typography>
                <Stack spacing={0.5}>
                  {relationBuckets.audio.slice(0, 12).map((item) => {
                    const url = item.trim();
                    const looksLikeUrl = /^https?:\/\//i.test(url);
                    return looksLikeUrl ? (
                      <a key={url} href={url} target="_blank" rel="noreferrer" className="text-sm text-sky-700 underline">
                        {url}
                      </a>
                    ) : (
                      <Typography key={url} variant="body2" color="text.secondary">{url}</Typography>
                    );
                  })}
                </Stack>
              </div>
            ) : null}
            {relationBuckets.others.length > 0 ? (
              <div>
                <Typography variant="subtitle2">Related</Typography>
                <Stack spacing={0.5}>
                  {relationBuckets.others.slice(0, 24).map((item) => (
                    <Typography key={item} variant="body2" color="text.secondary">{item}</Typography>
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
