# Dictionary Data Contract

## Source DB

- Path: `/win/linux/data/wiktionary/wiktionary.sqlite`
- Size observed: ~2.3 GB
- Required tables:
- `pages`
- `definitions`
- `relations`
- `lemma_aliases`

## Required Columns

- `pages`: `id`, `title`
- `definitions`: `id`, `page_id`, `language`, `def_order`, `definition_text`
- `relations`: `page_id`, `language`, `relation_type`, `rel_order`, `target_term`
- `lemma_aliases`: `page_id`, `alias`, `language`

If any required table/column is missing, Dictionary commands return a schema mismatch error.

## Query Behavior

### Language list

- SQL shape:
- `SELECT DISTINCT language FROM definitions ... ORDER BY language`
- Notes:
- Empty/whitespace languages are filtered.

### Search (`dictionary_search`)

- Search mode: prefix (`query%`)
- Sources:
- `pages.title LIKE ?`
- `lemma_aliases.alias LIKE ?`
- Language filter:
- optional, case-insensitive (`LOWER(...) = LOWER(?)`)
- Merge semantics:
- title hits and alias hits are `UNION` merged.
- Sorting:
- title-source rows before alias-source rows, then lexical by word.
- Limit:
- clamped to `[1, 500]`, default from config (`max_results`).

### Entry (`dictionary_entry`)

- Resolution order:
- by explicit `id` (page id), or
- exact title/alias match (`LOWER(...) = LOWER(?)`).
- If language is omitted:
- picks the highest-definition-count language for the page.
- Definition ordering:
- `ORDER BY def_order ASC`.
- Relation mapping:
- `pron*|ipa|phon*` -> pronunciation
- `pos|part_of_speech|word_class|grammatical*` -> part of speech
- `etym*|origin` -> etymology
- `example|usage` -> examples
- everything else -> notes (`<relation_type>: <target_term>`)

## Nullability / Fallback Rules

- `DictionarySearchHit.language` can be null only when upstream data is missing; normal path uses `definitions.language`.
- `DictionaryEntry.id` is page id and expected present in native schema.
- `part_of_speech`, `pronunciation`, `etymology` are optional; derived from `relations`.
- `definitions`, `senses`, `pronunciations`, `examples`, `notes`, `metadata` default to empty arrays if no data.

## Performance Baseline (local sample)

Measured with direct `sqlite3` calls against the current DB:

- `COUNT(*) definitions for English`: `0.011s`
- Prefix search on `pages.title LIKE 'rivet%' LIMIT 20`: `0.024s`
- Prefix search on `lemma_aliases.alias LIKE 'rivet%' LIMIT 20`: `0.429s`

Notes:

- `lemma_aliases.alias LIKE 'prefix%'` is materially slower than `pages.title` in this dataset.
- Existing indexes:
- `idx_pages_title`
- `idx_definitions_language`
- `idx_definitions_page`
- `idx_aliases_norm`
- `idx_aliases_page`

## Runtime Safety / Limits

- Connection mode: read-only
- `PRAGMA query_only = ON`
- Busy timeout: `2s`
- Slow-query warning threshold in backend tracing: `>= 250ms`
