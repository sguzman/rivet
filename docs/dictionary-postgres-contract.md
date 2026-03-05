# Dictionary Postgres Contract

## Backend

Rivet dictionary now reads from PostgreSQL only.

Default connection shape (from `rivet.toml`):

- `dictionary.postgres.host`
- `dictionary.postgres.port`
- `dictionary.postgres.user`
- `dictionary.postgres.password`
- `dictionary.postgres.database`
- `dictionary.postgres.schema`
- `dictionary.postgres.sslmode` (`disable` currently)
- `dictionary.postgres.connect_timeout_secs`
- `dictionary.postgres.max_connection_retries`
- `dictionary.postgres.retry_backoff_ms`

## Required Schema

Within configured schema (for example `dictionary`):

- `pages`
- `definitions`
- `relations`
- `lemma_aliases`

Required columns:

- `pages.id`, `pages.title`
- `definitions.id`, `definitions.page_id`, `definitions.language`, `definitions.def_order`, `definitions.definition_text`
- `relations.page_id`, `relations.language`, `relations.relation_type`, `relations.rel_order`, `relations.target_term`
- `lemma_aliases.page_id`, `lemma_aliases.alias`, `lemma_aliases.language`

Optional FTS table:

- `page_fts` (if present, used for `search_mode = "fts"`)

## Command Semantics

### `dictionary_languages`

- Returns distinct `definitions.language` values.
- Falls back to configured `default_language` only when no rows are present.

### `dictionary_search`

- Modes: `exact`, `prefix`, `fuzzy`, `fts`.
- `fts` degrades to `fuzzy` if `page_fts` is missing or query path fails.
- Maintains morphology fallback and language token normalization.
- Language token behavior:
- Exact language match preferred.
- ISO-like token mapping supported for common values (`en -> English`, `es -> Spanish`, etc.).
- If unresolved, language filter is dropped and a warning is returned.

### `dictionary_entry`

- Resolves by `id` or `word` (title first, alias second).
- Preserves structured output fields used by UI:
- pronunciations, POS, etymology, senses, examples, notes, metadata.

## Safety and Observability

- Bounded Postgres connection pool.
- Connection retries with backoff at startup/first-use.
- Structured tracing for command start/success/failure and slow-query warnings.
- Credential value is not emitted in logs.
