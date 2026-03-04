use std::time::Instant;

use rivet_gui_shared::{
  DictionaryEntry,
  DictionaryEntryArgs,
  DictionaryMeta,
  DictionaryPronunciation,
  DictionarySearchArgs,
  DictionarySearchHit,
  DictionarySearchResult,
  DictionarySense
};
use rusqlite::{
  Connection,
  OpenFlags,
  OptionalExtension,
  params
};

const DEFAULT_DICTIONARY_PATH: &str =
  "/win/linux/data/wiktionary/wiktionary.sqlite";
const DEFAULT_SEARCH_LIMIT: u32 = 100;
const MAX_SEARCH_LIMIT: u32 = 500;
const SQLITE_BUSY_TIMEOUT_SECS: u64 = 2;
const SLOW_QUERY_WARN_MS: u128 = 250;
const SQLITE_CACHE_SIZE_KIB: i64 = 16 * 1024;
const SEARCH_LIST_SQL: &str =
  "WITH title_hits AS (
      SELECT p.id AS page_id,
             p.title AS word,
             d.language AS language,
             (SELECT d2.definition_text
              FROM definitions d2
              WHERE d2.page_id = p.id
                AND d2.language = d.language
              ORDER BY d2.def_order ASC
              LIMIT 1) AS summary,
             0 AS source_rank
      FROM pages p
      JOIN definitions d ON d.page_id = p.id
      WHERE (
              (?1 = 'exact' AND LOWER(p.title) = LOWER(?2))
           OR (?1 = 'prefix' AND p.title LIKE ?3)
           OR (?1 = 'fuzzy' AND (p.title LIKE ?4 OR EXISTS(
                SELECT 1 FROM definitions d3
                WHERE d3.page_id = p.id
                  AND d3.language = d.language
                  AND d3.normalized_text LIKE ?4
              )))
            )
        AND (?5 IS NULL OR LOWER(d.language) = LOWER(?5))
      GROUP BY p.id, p.title, d.language
    ),
    alias_hits AS (
      SELECT la.page_id AS page_id,
             la.alias AS word,
             COALESCE(la.language, d.language) AS language,
             (SELECT d2.definition_text
              FROM definitions d2
              WHERE d2.page_id = la.page_id
                AND d2.language = COALESCE(la.language, d.language)
              ORDER BY d2.def_order ASC
              LIMIT 1) AS summary,
             1 AS source_rank
      FROM lemma_aliases la
      JOIN definitions d ON d.page_id = la.page_id
      WHERE (
              (?1 = 'exact' AND LOWER(la.alias) = LOWER(?2))
           OR (?1 = 'prefix' AND la.alias LIKE ?3)
           OR (?1 = 'fuzzy' AND (la.alias LIKE ?4 OR la.normalized_alias LIKE ?4))
            )
        AND (?5 IS NULL OR LOWER(COALESCE(la.language, d.language)) = LOWER(?5))
      GROUP BY la.page_id, la.alias, COALESCE(la.language, d.language)
    ),
    merged AS (
      SELECT page_id, word, language, summary, source_rank FROM title_hits
      UNION
      SELECT page_id, word, language, summary, source_rank FROM alias_hits
    )
    SELECT page_id, word, language, summary
    FROM merged
    ORDER BY source_rank ASC, word COLLATE NOCASE ASC
    LIMIT ?6";
const SEARCH_COUNT_SQL: &str =
  "WITH title_hits AS (
      SELECT p.id AS page_id,
             p.title AS word,
             d.language AS language
      FROM pages p
      JOIN definitions d ON d.page_id = p.id
      WHERE (
              (?1 = 'exact' AND LOWER(p.title) = LOWER(?2))
           OR (?1 = 'prefix' AND p.title LIKE ?3)
           OR (?1 = 'fuzzy' AND (p.title LIKE ?4 OR EXISTS(
                SELECT 1 FROM definitions d3
                WHERE d3.page_id = p.id
                  AND d3.language = d.language
                  AND d3.normalized_text LIKE ?4
              )))
            )
        AND (?5 IS NULL OR LOWER(d.language) = LOWER(?5))
      GROUP BY p.id, p.title, d.language
    ),
    alias_hits AS (
      SELECT la.page_id AS page_id,
             la.alias AS word,
             COALESCE(la.language, d.language) AS language
      FROM lemma_aliases la
      JOIN definitions d ON d.page_id = la.page_id
      WHERE (
              (?1 = 'exact' AND LOWER(la.alias) = LOWER(?2))
           OR (?1 = 'prefix' AND la.alias LIKE ?3)
           OR (?1 = 'fuzzy' AND (la.alias LIKE ?4 OR la.normalized_alias LIKE ?4))
            )
        AND (?5 IS NULL OR LOWER(COALESCE(la.language, d.language)) = LOWER(?5))
      GROUP BY la.page_id, la.alias, COALESCE(la.language, d.language)
    ),
    merged AS (
      SELECT page_id, word, language FROM title_hits
      UNION
      SELECT page_id, word, language FROM alias_hits
    )
    SELECT COUNT(1) FROM merged";

#[derive(Debug, serde::Deserialize)]
struct DictionaryConfig {
  enabled:          Option<bool>,
  sqlite_path:      Option<String>,
  default_language: Option<String>,
  max_results:      Option<u32>,
  search_mode:      Option<String>
}

#[derive(Debug, Clone)]
struct DictionarySettings {
  enabled:          bool,
  sqlite_path:      std::path::PathBuf,
  default_language: Option<String>,
  max_results:      u32,
  search_mode:      String
}

#[derive(Debug, Clone)]
struct NativeHit {
  page_id:  i64,
  word:     String,
  language: String,
  summary:  Option<String>
}

#[derive(Debug, Clone)]
struct RelationBuckets {
  pronunciation: Vec<String>,
  part_of_speech: Vec<String>,
  etymology:     Vec<String>,
  examples:      Vec<String>,
  notes:         Vec<String>,
  metadata:      Vec<DictionaryMeta>
}

fn normalized_language(
  value: Option<String>
) -> Option<String> {
  value.and_then(|raw| {
    let trimmed = raw.trim();
    if trimmed.is_empty()
      || trimmed == "*"
      || trimmed.eq_ignore_ascii_case(
        "all"
      )
    {
      None
    } else {
      Some(trimmed.to_string())
    }
  })
}

fn language_exists(
  conn: &Connection,
  language: &str
) -> anyhow::Result<bool> {
  let exists: i64 = conn.query_row(
    "SELECT EXISTS(
       SELECT 1 FROM definitions
       WHERE LOWER(language) = LOWER(?1)
       LIMIT 1
     )",
    params![language],
    |row| row.get(0),
  )?;
  Ok(exists != 0)
}

fn map_iso_language(
  token: &str
) -> Option<&'static str> {
  match token.trim().to_ascii_lowercase().as_str()
  {
    "en" => Some("English"),
    "es" => Some("Spanish"),
    "fr" => Some("French"),
    "de" => Some("German"),
    "it" => Some("Italian"),
    "pt" => Some("Portuguese"),
    "ru" => Some("Russian"),
    "pl" => Some("Polish"),
    "fi" => Some("Finnish"),
    "nl" => Some("Dutch"),
    "zh" => Some("Chinese"),
    "ja" => Some("Japanese"),
    "ko" => Some("Korean"),
    "sv" => Some("Swedish"),
    "tr" => Some("Turkish"),
    "vi" => Some("Vietnamese"),
    "la" => Some("Latin"),
    _ => None,
  }
}

fn resolve_language_filter(
  conn: &Connection,
  language: Option<String>
) -> anyhow::Result<(
  Option<String>,
  Vec<String>,
)> {
  let Some(token) = language else {
    return Ok((None, vec![]));
  };
  if language_exists(conn, &token)? {
    return Ok((Some(token), vec![]));
  }

  let mut warnings = Vec::<String>::new();
  if let Some(mapped) = map_iso_language(
    &token
  ) && language_exists(conn, mapped)?
  {
    warnings.push(format!(
      "language filter `{token}` mapped to `{mapped}`"
    ));
    return Ok((
      Some(mapped.to_string()),
      warnings,
    ));
  }

  warnings.push(format!(
    "language filter `{token}` has no matches; falling back to all languages"
  ));
  Ok((None, warnings))
}

fn normalize_for_search(
  value: &str
) -> String {
  value.trim().to_ascii_lowercase()
}

fn normalize_search_mode(
  value: Option<String>
) -> String {
  let Some(raw) = value else {
    return "prefix".to_string();
  };
  let mode = raw
    .trim()
    .to_ascii_lowercase();
  if mode == "exact"
    || mode == "prefix"
    || mode == "fuzzy"
    || mode == "fts"
  {
    mode
  } else {
    "prefix".to_string()
  }
}

fn lemma_candidates(
  query: &str
) -> Vec<String> {
  let token =
    query.trim().to_ascii_lowercase();
  if token.len() < 4 {
    return vec![];
  }
  let mut out = Vec::<String>::new();
  if token.ends_with("ies")
    && token.len() > 4
  {
    out.push(format!(
      "{}y",
      &token[..token.len() - 3]
    ));
  }
  if token.ends_with("es")
    && token.len() > 4
  {
    out.push(
      token[..token.len() - 2].to_string()
    );
  }
  if token.ends_with('s')
    && token.len() > 3
  {
    out.push(
      token[..token.len() - 1].to_string()
    );
  }
  if token.ends_with("ing")
    && token.len() > 5
  {
    let stem =
      token[..token.len() - 3].to_string();
    out.push(stem.clone());
    if let Some(last) = stem.chars().last()
    {
      out.push(format!("{stem}{last}"));
    }
    out.push(format!("{stem}e"));
  }
  if token.ends_with("ed")
    && token.len() > 4
  {
    let stem =
      token[..token.len() - 2].to_string();
    out.push(stem.clone());
    out.push(format!("{stem}e"));
  }
  out.retain(|candidate| {
    !candidate.is_empty()
      && candidate != &token
  });
  dedupe_strings(out)
}

fn dedupe_strings(
  values: Vec<String>
) -> Vec<String> {
  let mut out = Vec::<String>::new();
  for value in values {
    let trimmed = value.trim();
    if trimmed.is_empty() {
      continue;
    }
    if out
      .iter()
      .any(|existing| {
        existing.eq_ignore_ascii_case(
          trimmed
        )
      })
    {
      continue;
    }
    out.push(trimmed.to_string());
  }
  out
}

fn classify_relation_type(
  relation_type: &str,
  target: &str,
  buckets: &mut RelationBuckets
) {
  let relation =
    relation_type.trim().to_ascii_lowercase();
  let term = target.trim();
  if term.is_empty() {
    return;
  }

  if relation.contains("pron")
    || relation.contains("ipa")
    || relation.contains("phon")
  {
    buckets.pronunciation.push(
      term.to_string()
    );
    buckets.metadata.push(DictionaryMeta {
      relation_type: relation_type
        .to_string(),
      target:        term.to_string()
    });
    return;
  }

  if relation == "pos"
    || relation.contains("part_of_speech")
    || relation.contains("word_class")
    || relation.contains("grammatical")
  {
    buckets
      .part_of_speech
      .push(term.to_string());
    buckets.metadata.push(DictionaryMeta {
      relation_type: relation_type
        .to_string(),
      target:        term.to_string()
    });
    return;
  }

  if relation.contains("etym")
    || relation.contains("origin")
  {
    buckets.etymology.push(
      term.to_string()
    );
    buckets.metadata.push(DictionaryMeta {
      relation_type: relation_type
        .to_string(),
      target:        term.to_string()
    });
    return;
  }

  if relation.contains("example")
    || relation.contains("usage")
  {
    buckets.examples.push(
      term.to_string()
    );
    buckets.metadata.push(DictionaryMeta {
      relation_type: relation_type
        .to_string(),
      target:        term.to_string()
    });
    return;
  }

  buckets
    .notes
    .push(format!("{relation_type}: {term}"));
  buckets.metadata.push(DictionaryMeta {
    relation_type: relation_type.to_string(),
    target:        term.to_string()
  });
}

fn relation_buckets(
  conn: &Connection,
  page_id: i64,
  language: &str
) -> anyhow::Result<RelationBuckets> {
  let mut stmt = conn.prepare(
    "SELECT relation_type, target_term FROM relations \
     WHERE page_id = ?1 AND language = ?2 \
     ORDER BY rel_order ASC, id ASC LIMIT 512"
  )?;
  let mut rows = stmt.query(params![
    page_id,
    language
  ])?;

  let mut buckets = RelationBuckets {
    pronunciation: vec![],
    part_of_speech: vec![],
    etymology: vec![],
    examples: vec![],
    notes: vec![],
    metadata: vec![]
  };

  while let Some(row) = rows.next()? {
    let relation_type: String =
      row.get(0)?;
    let target: String = row.get(1)?;
    classify_relation_type(
      &relation_type,
      &target,
      &mut buckets,
    );
  }

  buckets.pronunciation =
    dedupe_strings(buckets.pronunciation);
  buckets.part_of_speech =
    dedupe_strings(buckets.part_of_speech);
  buckets.etymology =
    dedupe_strings(buckets.etymology);
  buckets.examples =
    dedupe_strings(buckets.examples);
  buckets.notes =
    dedupe_strings(buckets.notes);

  Ok(buckets)
}

fn table_exists(
  conn: &Connection,
  table: &str
) -> anyhow::Result<bool> {
  let exists = conn.query_row(
    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
    params![table],
    |row| row.get::<_, i64>(0),
  )?;
  Ok(exists != 0)
}

fn table_columns(
  conn: &Connection,
  table: &str
) -> anyhow::Result<Vec<String>> {
  let pragma =
    format!("PRAGMA table_info({table})");
  let mut stmt = conn.prepare(&pragma)?;
  let columns = stmt
    .query_map([], |row| {
      row.get::<_, String>(1)
    })?
    .collect::<Result<Vec<_>, _>>()?;
  Ok(columns)
}

fn ensure_required_schema(
  conn: &Connection
) -> anyhow::Result<()> {
  let required_tables = [
    "pages",
    "definitions",
    "relations",
    "lemma_aliases"
  ];

  for table in required_tables {
    if !table_exists(conn, table)? {
      anyhow::bail!(
        "dictionary schema mismatch: missing table `{table}`"
      );
    }
  }

  let page_cols = table_columns(conn, "pages")?;
  let def_cols =
    table_columns(conn, "definitions")?;
  let rel_cols =
    table_columns(conn, "relations")?;
  let alias_cols =
    table_columns(conn, "lemma_aliases")?;

  let page_req = ["id", "title"];
  let def_req = [
    "id",
    "page_id",
    "language",
    "def_order",
    "definition_text"
  ];
  let rel_req = [
    "page_id",
    "language",
    "relation_type",
    "rel_order",
    "target_term"
  ];
  let alias_req = [
    "page_id",
    "alias",
    "language"
  ];

  for col in page_req {
    if !page_cols.iter().any(|c| c == col)
    {
      anyhow::bail!(
        "dictionary schema mismatch: missing pages.{col}"
      );
    }
  }
  for col in def_req {
    if !def_cols.iter().any(|c| c == col)
    {
      anyhow::bail!(
        "dictionary schema mismatch: missing definitions.{col}"
      );
    }
  }
  for col in rel_req {
    if !rel_cols.iter().any(|c| c == col)
    {
      anyhow::bail!(
        "dictionary schema mismatch: missing relations.{col}"
      );
    }
  }
  for col in alias_req {
    if !alias_cols.iter().any(|c| c == col)
    {
      anyhow::bail!(
        "dictionary schema mismatch: missing lemma_aliases.{col}"
      );
    }
  }

  Ok(())
}

fn open_dictionary_connection(
  path: &std::path::Path
) -> anyhow::Result<Connection> {
  let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
    | OpenFlags::SQLITE_OPEN_URI
    | OpenFlags::SQLITE_OPEN_NO_MUTEX;
  let conn = Connection::open_with_flags(
    path, flags,
  )
  .with_context(|| {
    format!(
      "failed to open dictionary sqlite DB at {}",
      path.display()
    )
  })?;

  conn
    .busy_timeout(std::time::Duration::from_secs(
      SQLITE_BUSY_TIMEOUT_SECS,
    ))
    .with_context(|| {
      format!(
        "failed to set busy timeout on dictionary DB at {}",
        path.display()
      )
    })?;

  conn
    .pragma_update(None, "query_only", "ON")
    .with_context(|| {
      format!(
        "failed to enforce query_only mode for dictionary DB at {}",
        path.display()
      )
    })?;

  // Keep temp structures on disk and bound page cache usage for very large dictionary DBs.
  conn
    .pragma_update(None, "temp_store", "FILE")
    .with_context(|| {
      format!(
        "failed to enforce temp_store=FILE for dictionary DB at {}",
        path.display()
      )
    })?;
  conn
    .pragma_update(
      None,
      "cache_size",
      format!("-{SQLITE_CACHE_SIZE_KIB}"),
    )
    .with_context(|| {
      format!(
        "failed to set cache_size for dictionary DB at {}",
        path.display()
      )
    })?;

  Ok(conn)
}

fn resolve_dictionary_settings(
) -> anyhow::Result<DictionarySettings> {
  let config_path =
    resolve_config_path("rivet.toml");
  let mut enabled = true;
  let mut sqlite_path =
    std::path::PathBuf::from(
      DEFAULT_DICTIONARY_PATH,
    );
  let mut default_language = None;
  let mut max_results =
    DEFAULT_SEARCH_LIMIT;
  let mut search_mode =
    "prefix".to_string();

  if config_path.is_file() {
    let raw = std::fs::read_to_string(
      &config_path,
    )
    .with_context(|| {
      format!(
        "failed to read {}",
        config_path.display()
      )
    })?;
    let value = toml::from_str::<toml::Value>(
      &raw,
    )
    .with_context(|| {
      format!(
        "failed to parse {}",
        config_path.display()
      )
    })?;
    if let Some(dictionary) = value
      .get("dictionary")
      .and_then(|entry| {
        entry.clone().try_into::<
          DictionaryConfig,
        >()
        .ok()
      })
    {
      if let Some(flag) =
        dictionary.enabled
      {
        enabled = flag;
      }
      if let Some(path) =
        dictionary.sqlite_path
      {
        sqlite_path =
          std::path::PathBuf::from(path);
      }
      default_language = normalized_language(
        dictionary.default_language,
      );
      if let Some(limit) =
        dictionary.max_results
      {
        max_results = limit.clamp(
          1,
          MAX_SEARCH_LIMIT,
        );
      }
      search_mode =
        normalize_search_mode(
          dictionary.search_mode
        );
    }
  }

  if let Ok(env_path) = std::env::var(
    "RIVET_DICTIONARY_SQLITE",
  ) {
    let trimmed = env_path.trim();
    if !trimmed.is_empty() {
      sqlite_path =
        std::path::PathBuf::from(trimmed);
    }
  }

  let sqlite_path = if sqlite_path
    .is_absolute()
  {
    sqlite_path
  } else if let Some(parent) =
    config_path.parent()
  {
    parent.join(sqlite_path)
  } else {
    sqlite_path
  };

  Ok(DictionarySettings {
    enabled,
    sqlite_path,
    default_language,
    max_results,
    search_mode
  })
}

fn ensure_dictionary_ready() -> anyhow::Result<(
  DictionarySettings,
  Connection,
)> {
  let settings =
    resolve_dictionary_settings()?;
  if !settings.enabled {
    anyhow::bail!(
      "dictionary is disabled in config ([dictionary].enabled = false)"
    );
  }
  if !settings.sqlite_path.is_file() {
    anyhow::bail!(
      "dictionary sqlite path missing: {}",
      settings.sqlite_path.display()
    );
  }

  let conn = open_dictionary_connection(
    &settings.sqlite_path,
  )?;
  ensure_required_schema(&conn)?;

  Ok((settings, conn))
}

fn log_if_slow(
  label: &str,
  started: Instant,
  request_id: &Option<String>,
  detail: &str
) {
  let elapsed_ms =
    started.elapsed().as_millis();
  if elapsed_ms >= SLOW_QUERY_WARN_MS {
    tracing::warn!(
      request_id = ?request_id,
      elapsed_ms,
      detail,
      "slow dictionary query: {label}"
    );
  }
}

fn dictionary_languages_native(
  settings: &DictionarySettings,
  conn: &Connection
) -> anyhow::Result<Vec<String>> {
  let mut stmt = conn.prepare(
    "SELECT DISTINCT language FROM definitions WHERE TRIM(language) <> '' ORDER BY language COLLATE NOCASE LIMIT 2048"
  )?;
  let mut values = stmt
    .query_map([], |row| {
      row.get::<_, String>(0)
    })?
    .collect::<Result<Vec<_>, _>>()?;
  values.retain(|value| {
    !value.trim().is_empty()
  });
  if values.is_empty()
    && let Some(default_language) =
      settings.default_language.as_ref()
  {
    values.push(default_language.clone());
  }
  Ok(values)
}

fn detect_fts_table(
  conn: &Connection
) -> anyhow::Result<Option<&'static str>> {
  if table_exists(
    conn,
    "definitions_fts",
  )? {
    return Ok(Some("definitions_fts"));
  }
  if table_exists(
    conn,
    "lemma_aliases_fts",
  )? {
    return Ok(Some("lemma_aliases_fts"));
  }
  if table_exists(conn, "pages_fts")? {
    return Ok(Some("pages_fts"));
  }
  Ok(None)
}

fn run_search_hits(
  conn: &Connection,
  mode: &str,
  query: &str,
  language: &Option<String>,
  limit: u32
) -> anyhow::Result<Vec<NativeHit>> {
  let exact = query.to_string();
  let prefix = format!("{query}%");
  let fuzzy = format!("%{query}%");
  let mut stmt =
    conn.prepare(SEARCH_LIST_SQL)?;
  let rows = stmt
    .query_map(
      params![
        mode,
        exact,
        prefix,
        fuzzy,
        language,
        i64::from(limit),
      ],
      |row| {
        Ok(NativeHit {
          page_id: row.get(0)?,
          word: row.get(1)?,
          language: row.get(2)?,
          summary:
            row.get::<_, Option<String>>(3)?,
        })
      },
    )?
    .collect::<Result<Vec<_>, _>>()?;
  Ok(rows)
}

fn run_search_count(
  conn: &Connection,
  mode: &str,
  query: &str,
  language: &Option<String>
) -> anyhow::Result<i64> {
  let exact = query.to_string();
  let prefix = format!("{query}%");
  let fuzzy = format!("%{query}%");
  let total: i64 = conn.query_row(
    SEARCH_COUNT_SQL,
    params![
      mode,
      exact,
      prefix,
      fuzzy,
      language,
    ],
    |row| row.get(0),
  )?;
  Ok(total)
}

fn run_fts_hits(
  conn: &Connection,
  table: &str,
  query: &str,
  language: &Option<String>,
  limit: u32
) -> anyhow::Result<Vec<NativeHit>> {
  let sql = if table == "definitions_fts" {
    "SELECT d.page_id, p.title, d.language,
            (SELECT d2.definition_text
             FROM definitions d2
             WHERE d2.page_id = d.page_id
               AND d2.language = d.language
             ORDER BY d2.def_order ASC
             LIMIT 1) AS summary
     FROM definitions_fts f
     JOIN definitions d ON d.id = f.rowid
     JOIN pages p ON p.id = d.page_id
     WHERE definitions_fts MATCH ?1
       AND (?2 IS NULL OR LOWER(d.language) = LOWER(?2))
     GROUP BY d.page_id, p.title, d.language
     ORDER BY p.title COLLATE NOCASE ASC
     LIMIT ?3"
  } else if table == "lemma_aliases_fts" {
    "SELECT la.page_id, la.alias, COALESCE(la.language, d.language),
            (SELECT d2.definition_text
             FROM definitions d2
             WHERE d2.page_id = la.page_id
               AND d2.language = COALESCE(la.language, d.language)
             ORDER BY d2.def_order ASC
             LIMIT 1) AS summary
     FROM lemma_aliases_fts f
     JOIN lemma_aliases la ON la.id = f.rowid
     JOIN definitions d ON d.page_id = la.page_id
     WHERE lemma_aliases_fts MATCH ?1
       AND (?2 IS NULL OR LOWER(COALESCE(la.language, d.language)) = LOWER(?2))
     GROUP BY la.page_id, la.alias, COALESCE(la.language, d.language)
     ORDER BY la.alias COLLATE NOCASE ASC
     LIMIT ?3"
  } else {
    "SELECT p.id, p.title, d.language,
            (SELECT d2.definition_text
             FROM definitions d2
             WHERE d2.page_id = p.id
               AND d2.language = d.language
             ORDER BY d2.def_order ASC
             LIMIT 1) AS summary
     FROM pages_fts f
     JOIN pages p ON p.id = f.rowid
     JOIN definitions d ON d.page_id = p.id
     WHERE pages_fts MATCH ?1
       AND (?2 IS NULL OR LOWER(d.language) = LOWER(?2))
     GROUP BY p.id, p.title, d.language
     ORDER BY p.title COLLATE NOCASE ASC
     LIMIT ?3"
  };

  let mut stmt = conn.prepare(sql)?;
  let rows = stmt
    .query_map(
      params![
        query,
        language,
        i64::from(limit),
      ],
      |row| {
        Ok(NativeHit {
          page_id: row.get(0)?,
          word: row.get(1)?,
          language: row.get(2)?,
          summary:
            row.get::<_, Option<String>>(3)?,
        })
      },
    )?
    .collect::<Result<Vec<_>, _>>()?;
  Ok(rows)
}

fn dictionary_search_native(
  settings: &DictionarySettings,
  conn: &Connection,
  args: DictionarySearchArgs,
  request_id: &Option<String>
) -> anyhow::Result<DictionarySearchResult> {
  let query = args.query.trim();
  if query.is_empty() {
    return Ok(DictionarySearchResult {
      query: query.to_string(),
      language: normalized_language(
        args.language,
      ),
      hits: vec![],
      total: 0,
      truncated: false,
      warnings: vec![
        "query is empty".to_string()
      ]
    });
  }

  let requested_language =
    normalized_language(args.language)
      .or(
        settings.default_language.clone()
      );
  let (
    language,
    mut language_warnings,
  ) = resolve_language_filter(
    conn,
    requested_language,
  )?;
  let search_mode =
    normalize_search_mode(args.mode)
      .trim()
      .to_string();
  let limit = args
    .limit
    .unwrap_or(settings.max_results)
    .clamp(1, MAX_SEARCH_LIMIT);
  let effective_mode = if search_mode
    .is_empty()
  {
    settings.search_mode.clone()
  } else {
    search_mode
  };
  let mut warnings = Vec::<String>::new();
  warnings.append(&mut language_warnings);

  let list_started = Instant::now();
  let mut rows = if effective_mode == "fts" {
    match detect_fts_table(conn)? {
      Some(table) => {
        match run_fts_hits(
          conn,
          table,
          query,
          &language,
          limit,
        ) {
          Ok(payload) => payload,
          Err(err) => {
            warnings.push(format!(
              "fts query unavailable ({err}); falling back to fuzzy search"
            ));
            run_search_hits(
              conn,
              "fuzzy",
              query,
              &language,
              limit,
            )?
          }
        }
      }
      None => {
        warnings.push(
          "fts mode requested but no fts virtual table found; falling back to fuzzy search"
            .to_string(),
        );
        run_search_hits(
          conn,
          "fuzzy",
          query,
          &language,
          limit,
        )?
      }
    }
  } else {
    run_search_hits(
      conn,
      &effective_mode,
      query,
      &language,
      limit,
    )?
  };
  log_if_slow(
    "search.list",
    list_started,
    request_id,
    "title+alias lookup",
  );

  let count_started = Instant::now();
  let mut total: i64 =
    if effective_mode == "fts" {
      rows.len() as i64
    } else {
      run_search_count(
        conn,
        &effective_mode,
        query,
        &language,
      )?
    };
  log_if_slow(
    "search.count",
    count_started,
    request_id,
    "count merged hits",
  );

  if rows.is_empty() {
    let candidates = lemma_candidates(query);
    if !candidates.is_empty() {
      let fallback_started =
        Instant::now();
      let mut fallback_rows =
        Vec::<NativeHit>::new();
      for candidate in &candidates {
        let candidate_hits =
          run_search_hits(
            conn,
            "exact",
            candidate,
            &language,
            limit,
          )?;
        for hit in candidate_hits {
          let exists = fallback_rows.iter().any(
            |entry| {
              entry.page_id == hit.page_id
                && entry
                  .language
                  .eq_ignore_ascii_case(
                    &hit.language
                  )
            },
          );
          if exists {
            continue;
          }
          fallback_rows.push(hit);
          if fallback_rows.len() >= limit as usize
          {
            break;
          }
        }
        if fallback_rows.len() >= limit as usize
        {
          break;
        }
      }
      if !fallback_rows.is_empty() {
        warnings.push(format!(
          "no direct matches; applied morphology fallback candidates: {}",
          candidates.join(", ")
        ));
        total = fallback_rows.len() as i64;
        rows = fallback_rows;
      }
      log_if_slow(
        "search.morphology_fallback",
        fallback_started,
        request_id,
        "lemma candidate search",
      );
    }
  }

  let enrich_started = Instant::now();
  let mut hits = Vec::<DictionarySearchHit>::new();
  for row in rows {
    let relation = relation_buckets(
      conn,
      row.page_id,
      &row.language,
    )?;
    hits.push(DictionarySearchHit {
      id: Some(row.page_id),
      word: row.word,
      language: Some(row.language),
      part_of_speech: relation
        .part_of_speech
        .first()
        .cloned(),
      pronunciation: relation
        .pronunciation
        .first()
        .cloned(),
      summary: row.summary,
      source_table: "pages/definitions/lemma_aliases"
        .to_string(),
      matched_by_prefix: effective_mode
        == "prefix",
    });
  }
  log_if_slow(
    "search.enrich",
    enrich_started,
    request_id,
    "relation enrichment",
  );

  Ok(DictionarySearchResult {
    query: query.to_string(),
    language,
    total: total.max(0) as u64,
    truncated: total.max(0) as u64 > hits.len() as u64,
    hits,
    warnings,
  })
}

fn dictionary_entry_native(
  settings: &DictionarySettings,
  conn: &Connection,
  args: DictionaryEntryArgs,
  request_id: &Option<String>
) -> anyhow::Result<Option<DictionaryEntry>> {
  let requested_language =
    normalized_language(args.language)
      .or(settings.default_language.clone());
  let (
    requested_language,
    _language_warnings,
  ) = resolve_language_filter(
    conn,
    requested_language,
  )?;
  let requested_word = args.word.and_then(
    |word| {
      let trimmed = word.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed.to_string())
      }
    },
  );

  if args.id.is_none()
    && requested_word.is_none()
  {
    anyhow::bail!(
      "dictionary_entry requires id or word"
    );
  }

  let resolve_started = Instant::now();
  let resolved: Option<(i64, String)> = if let Some(page_id) = args.id {
    if let Some(language) = requested_language.as_ref() {
      Some((page_id, language.clone()))
    } else {
      let best = conn
        .query_row(
          "SELECT language FROM definitions WHERE page_id = ?1 GROUP BY language ORDER BY COUNT(*) DESC, language ASC LIMIT 1",
          params![page_id],
          |row| row.get::<_, String>(0),
        )
        .optional()?;
      best.map(|language| (page_id, language))
    }
  } else {
    let word = requested_word.clone().unwrap_or_default();
    conn
      .query_row(
        "WITH title_match AS (
            SELECT p.id AS page_id,
                   d.language AS language,
                   0 AS rank
            FROM pages p
            JOIN definitions d ON d.page_id = p.id
            WHERE LOWER(p.title) = LOWER(?1)
              AND (?2 IS NULL OR LOWER(d.language) = LOWER(?3))
            GROUP BY p.id, d.language
          ),
          alias_match AS (
            SELECT la.page_id AS page_id,
                   COALESCE(la.language, d.language) AS language,
                   1 AS rank
            FROM lemma_aliases la
            JOIN definitions d ON d.page_id = la.page_id
            WHERE LOWER(la.alias) = LOWER(?4)
              AND (?5 IS NULL OR LOWER(COALESCE(la.language, d.language)) = LOWER(?6))
            GROUP BY la.page_id, COALESCE(la.language, d.language)
          ),
          merged AS (
            SELECT page_id, language, rank FROM title_match
            UNION
            SELECT page_id, language, rank FROM alias_match
          )
          SELECT page_id, language
          FROM merged
          ORDER BY rank ASC, page_id ASC
          LIMIT 1",
        params![
          word,
          requested_language,
          requested_language,
          word,
          requested_language,
          requested_language,
        ],
        |row| {
          Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
          ))
        },
      )
      .optional()?
  };
  log_if_slow(
    "entry.resolve",
    resolve_started,
    request_id,
    "resolve page+language",
  );

  let Some((page_id, language)) = resolved
  else {
    return Ok(None);
  };

  let word_started = Instant::now();
  let word: String = conn.query_row(
    "SELECT title FROM pages WHERE id = ?1 LIMIT 1",
    params![page_id],
    |row| row.get(0),
  )?;
  log_if_slow(
    "entry.word",
    word_started,
    request_id,
    "fetch page title",
  );

  let defs_started = Instant::now();
  let mut def_stmt = conn.prepare(
    "SELECT definition_text FROM definitions WHERE page_id = ?1 AND language = ?2 ORDER BY def_order ASC"
  )?;
  let definitions = def_stmt
    .query_map(
      params![page_id, language],
      |row| row.get::<_, String>(0),
    )?
    .collect::<Result<Vec<_>, _>>()?;
  log_if_slow(
    "entry.definitions",
    defs_started,
    request_id,
    "fetch ordered definitions",
  );

  let rel_started = Instant::now();
  let relation = relation_buckets(
    conn,
    page_id,
    &language,
  )?;
  log_if_slow(
    "entry.relations",
    rel_started,
    request_id,
    "fetch relations",
  );

  let senses = definitions
    .iter()
    .enumerate()
    .map(|(idx, text)| DictionarySense {
      order: idx as u32 + 1,
      text:  text.clone()
    })
    .collect::<Vec<_>>();
  let pronunciations = relation
    .pronunciation
    .iter()
    .map(|text| DictionaryPronunciation {
      text:   text.clone(),
      system: None
    })
    .collect::<Vec<_>>();

  Ok(Some(DictionaryEntry {
    id: Some(page_id),
    word,
    language: Some(language),
    part_of_speech: relation
      .part_of_speech
      .first()
      .cloned(),
    pronunciation: relation
      .pronunciation
      .first()
      .cloned(),
    etymology: if relation.etymology.is_empty()
    {
      None
    } else {
      Some(relation.etymology.join(" | "))
    },
    definitions,
    examples: relation.examples,
    notes: relation.notes,
    metadata: relation.metadata,
    source_table: "pages/definitions/relations/lemma_aliases"
      .to_string(),
    senses,
    pronunciations
  }))
}

#[tauri::command]
#[tracing::instrument(fields(request_id = ?request_id))]
pub async fn dictionary_languages(
  request_id: Option<String>
) -> Result<Vec<String>, String> {
  let started = Instant::now();
  tracing::info!(
    request_id = ?request_id,
    "dictionary_languages command invoked"
  );

  let result =
    (|| -> anyhow::Result<Vec<String>> {
      let (settings, conn) =
        ensure_dictionary_ready()?;
      dictionary_languages_native(
        &settings,
        &conn,
      )
    })();

  let elapsed_ms =
    started.elapsed().as_millis();
  if let Err(err) = result.as_ref() {
    tracing::error!(
      request_id = ?request_id,
      elapsed_ms,
      error = %err,
      "dictionary_languages command failed"
    );
  } else {
    tracing::info!(
      request_id = ?request_id,
      elapsed_ms,
      count = result.as_ref().map_or(0, |langs| langs.len()),
      "dictionary_languages command completed"
    );
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[tracing::instrument(skip(args), fields(request_id = ?request_id, query_len = args.query.len(), language = ?args.language))]
pub async fn dictionary_search(
  args: DictionarySearchArgs,
  request_id: Option<String>
) -> Result<DictionarySearchResult, String> {
  let started = Instant::now();
  tracing::info!(
    request_id = ?request_id,
    query_len = args.query.len(),
    language = ?args.language,
    "dictionary_search command invoked"
  );

  let result =
    (|| -> anyhow::Result<
      DictionarySearchResult,
    > {
      let (settings, conn) =
        ensure_dictionary_ready()?;
      dictionary_search_native(
        &settings,
        &conn,
        DictionarySearchArgs {
          language: args.language,
          query: normalize_for_search(
            &args.query,
          ),
          limit: args.limit,
          mode: args.mode
        },
        &request_id,
      )
    })();

  let elapsed_ms =
    started.elapsed().as_millis();
  if let Err(err) = result.as_ref() {
    tracing::error!(
      request_id = ?request_id,
      elapsed_ms,
      error = %err,
      "dictionary_search command failed"
    );
  } else if let Ok(payload) =
    result.as_ref()
  {
    tracing::info!(
      request_id = ?request_id,
      elapsed_ms,
      hits = payload.hits.len(),
      total = payload.total,
      "dictionary_search command completed"
    );
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[tracing::instrument(skip(args), fields(request_id = ?request_id, id = ?args.id, language = ?args.language, has_word = args.word.as_ref().is_some_and(|word| !word.trim().is_empty())))]
pub async fn dictionary_entry(
  args: DictionaryEntryArgs,
  request_id: Option<String>
) -> Result<Option<DictionaryEntry>, String>
{
  let started = Instant::now();
  tracing::info!(
    request_id = ?request_id,
    id = ?args.id,
    language = ?args.language,
    "dictionary_entry command invoked"
  );

  let result = (|| -> anyhow::Result<
    Option<DictionaryEntry>,
  > {
    let (settings, conn) =
      ensure_dictionary_ready()?;
    dictionary_entry_native(
      &settings,
      &conn,
      args,
      &request_id,
    )
  })();

  let elapsed_ms =
    started.elapsed().as_millis();
  if let Err(err) = result.as_ref() {
    tracing::error!(
      request_id = ?request_id,
      elapsed_ms,
      error = %err,
      "dictionary_entry command failed"
    );
  } else {
    tracing::debug!(
      request_id = ?request_id,
      elapsed_ms,
      found = result
        .as_ref()
        .ok()
        .and_then(|entry| entry.as_ref())
        .is_some(),
      "dictionary_entry command completed"
    );
  }

  result.map_err(err_to_string)
}

#[cfg(test)]
mod dictionary_tests {
  use std::{
    fs,
    path::PathBuf,
    time::{
      SystemTime,
      UNIX_EPOCH
    }
  };

  use super::{
    DictionarySettings,
    RelationBuckets,
    classify_relation_type,
    dictionary_entry_native,
    dictionary_languages_native,
    dictionary_search_native,
    dedupe_strings,
    ensure_required_schema,
    lemma_candidates,
    normalize_search_mode,
    normalized_language,
    resolve_language_filter
  };
  use rivet_gui_shared::{
    DictionaryEntryArgs,
    DictionarySearchArgs
  };
  use rusqlite::Connection;

  fn fixture_db_path() -> PathBuf {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("valid unix time")
      .as_nanos();
    std::env::temp_dir()
      .join(format!(
        "rivet-dictionary-test-{nonce}.sqlite"
      ))
  }

  fn build_fixture_db() -> (PathBuf, Connection)
  {
    let path = fixture_db_path();
    let conn = Connection::open(&path)
      .expect("open fixture sqlite");
    conn.execute_batch(
      "
      CREATE TABLE pages (
        id INTEGER PRIMARY KEY,
        title TEXT NOT NULL
      );
      CREATE TABLE definitions (
        id INTEGER PRIMARY KEY,
        page_id INTEGER NOT NULL,
        language TEXT NOT NULL,
        def_order INTEGER NOT NULL,
        definition_text TEXT NOT NULL,
        normalized_text TEXT
      );
      CREATE TABLE relations (
        id INTEGER PRIMARY KEY,
        page_id INTEGER NOT NULL,
        language TEXT NOT NULL,
        relation_type TEXT NOT NULL,
        rel_order INTEGER NOT NULL,
        target_term TEXT NOT NULL
      );
      CREATE TABLE lemma_aliases (
        id INTEGER PRIMARY KEY,
        page_id INTEGER NOT NULL,
        alias TEXT NOT NULL,
        language TEXT,
        normalized_alias TEXT
      );
      ",
    )
    .expect("create fixture schema");

    conn.execute(
      "INSERT INTO pages (id, title) VALUES (?1, ?2)",
      rusqlite::params![1_i64, "anchor"],
    )
    .expect("insert page anchor");
    conn.execute(
      "INSERT INTO pages (id, title) VALUES (?1, ?2)",
      rusqlite::params![2_i64, "anchored"],
    )
    .expect("insert page anchored");
    conn.execute(
      "INSERT INTO pages (id, title) VALUES (?1, ?2)",
      rusqlite::params![3_i64, "ancla"],
    )
    .expect("insert page ancla");

    conn.execute(
      "INSERT INTO definitions (id, page_id, language, def_order, definition_text, normalized_text) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
      rusqlite::params![1_i64, 1_i64, "en", 1_i64, "A heavy object used to secure a vessel.", "a heavy object used to secure a vessel"],
    )
    .expect("insert definition en anchor");
    conn.execute(
      "INSERT INTO definitions (id, page_id, language, def_order, definition_text, normalized_text) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
      rusqlite::params![2_i64, 2_i64, "en", 1_i64, "Past tense of anchor.", "past tense of anchor"],
    )
    .expect("insert definition en anchored");
    conn.execute(
      "INSERT INTO definitions (id, page_id, language, def_order, definition_text, normalized_text) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
      rusqlite::params![3_i64, 3_i64, "es", 1_i64, "Objeto pesado para fijar una embarcación.", "objeto pesado para fijar una embarcacion"],
    )
    .expect("insert definition es ancla");

    conn.execute(
      "INSERT INTO relations (id, page_id, language, relation_type, rel_order, target_term) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
      rusqlite::params![1_i64, 1_i64, "en", "pronunciation_ipa", 1_i64, "/ˈæŋ.kɚ/"],
    )
    .expect("insert relation pronunciation");
    conn.execute(
      "INSERT INTO relations (id, page_id, language, relation_type, rel_order, target_term) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
      rusqlite::params![2_i64, 1_i64, "en", "part_of_speech", 2_i64, "noun"],
    )
    .expect("insert relation pos");
    conn.execute(
      "INSERT INTO relations (id, page_id, language, relation_type, rel_order, target_term) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
      rusqlite::params![3_i64, 1_i64, "en", "synonym", 3_i64, "grapnel"],
    )
    .expect("insert relation synonym");

    conn.execute(
      "INSERT INTO lemma_aliases (id, page_id, alias, language, normalized_alias) VALUES (?1, ?2, ?3, ?4, ?5)",
      rusqlite::params![1_i64, 1_i64, "anchors", "en", "anchors"],
    )
    .expect("insert alias anchors");

    (path, conn)
  }

  fn fixture_settings(path: &std::path::Path) -> DictionarySettings {
    DictionarySettings {
      enabled: true,
      sqlite_path: path.to_path_buf(),
      default_language: Some("en".to_string()),
      max_results: 100,
      search_mode: "prefix".to_string(),
    }
  }

  #[test]
  fn normalized_language_handles_all() {
    assert_eq!(
      normalized_language(Some(
        "all".to_string()
      )),
      None
    );
    assert_eq!(
      normalized_language(Some(
        "en".to_string()
      )),
      Some("en".to_string())
    );
  }

  #[test]
  fn dedupe_strings_is_case_insensitive()
  {
    let out = dedupe_strings(vec![
      "Alpha".to_string(),
      "alpha".to_string(),
      "Beta".to_string(),
      "".to_string(),
    ]);
    assert_eq!(
      out,
      vec![
        "Alpha".to_string(),
        "Beta".to_string()
      ]
    );
  }

  #[test]
  fn normalize_search_mode_defaults_to_prefix()
  {
    assert_eq!(
      normalize_search_mode(None),
      "prefix".to_string()
    );
    assert_eq!(
      normalize_search_mode(Some(
        "nope".to_string()
      )),
      "prefix".to_string()
    );
    assert_eq!(
      normalize_search_mode(Some(
        "fuzzy".to_string()
      )),
      "fuzzy".to_string()
    );
    assert_eq!(
      normalize_search_mode(Some(
        "fts".to_string()
      )),
      "fts".to_string()
    );
  }

  #[test]
  fn lemma_candidates_reduces_inflections()
  {
    let candidates =
      lemma_candidates("anchoring");
    assert!(
      candidates
        .iter()
        .any(|candidate| candidate == "anchor")
    );
  }

  #[test]
  fn resolve_language_filter_maps_iso_codes()
  {
    let conn = Connection::open_in_memory()
      .expect("open memory db");
    conn
      .execute_batch(
        "CREATE TABLE definitions (language TEXT NOT NULL);
         INSERT INTO definitions(language) VALUES ('English');",
      )
      .expect(
        "create language fixture"
      );
    let (resolved, warnings) =
      resolve_language_filter(
        &conn,
        Some("en".to_string()),
      )
      .expect(
        "resolve language filter"
      );
    assert_eq!(
      resolved.as_deref(),
      Some("English")
    );
    assert!(
      warnings
        .iter()
        .any(|w| w.contains("mapped"))
    );
  }

  #[test]
  fn classify_relation_type_buckets_synonym()
  {
    let mut buckets =
      RelationBuckets {
        pronunciation: vec![],
        part_of_speech: vec![],
        etymology: vec![],
        examples: vec![],
        notes: vec![],
        metadata: vec![]
      };
    classify_relation_type(
      "synonym",
      "anchor",
      &mut buckets,
    );
    assert_eq!(buckets.notes.len(), 1);
    assert_eq!(buckets.metadata.len(), 1);
    assert_eq!(
      buckets.metadata[0].relation_type,
      "synonym"
    );
    assert_eq!(
      buckets.metadata[0].target,
      "anchor"
    );
  }

  #[test]
  fn integration_fixture_languages_search_and_entry() {
    let (path, conn) = build_fixture_db();
    ensure_required_schema(&conn)
      .expect("fixture schema valid");
    let settings = fixture_settings(&path);

    let languages =
      dictionary_languages_native(
        &settings,
        &conn,
      )
      .expect("list languages");
    assert_eq!(
      languages,
      vec![
        "en".to_string(),
        "es".to_string()
      ]
    );

    let search =
      dictionary_search_native(
        &settings,
        &conn,
        DictionarySearchArgs {
          language: Some("en".to_string()),
          query: "anch".to_string(),
          limit: Some(10),
          mode: Some("prefix".to_string()),
        },
        &None,
      )
      .expect("search fixture");
    assert!(!search.hits.is_empty());
    assert!(
      search
        .hits
        .iter()
        .any(|hit| hit.word == "anchor")
    );

    let entry =
      dictionary_entry_native(
        &settings,
        &conn,
        DictionaryEntryArgs {
          id: Some(1),
          word: None,
          language: Some("en".to_string()),
        },
        &None,
      )
      .expect("load entry")
      .expect("entry present");
    assert_eq!(entry.word, "anchor");
    assert_eq!(
      entry.pronunciation.as_deref(),
      Some("/ˈæŋ.kɚ/")
    );
    assert_eq!(
      entry.part_of_speech.as_deref(),
      Some("noun")
    );
    assert!(
      entry
        .metadata
        .iter()
        .any(|m| m.relation_type == "synonym")
    );

    drop(conn);
    let _ = fs::remove_file(path);
  }

  #[test]
  fn integration_fixture_supports_alias_and_fuzzy() {
    let (path, conn) = build_fixture_db();
    ensure_required_schema(&conn)
      .expect("fixture schema valid");
    let settings = fixture_settings(&path);

    let alias_hit =
      dictionary_search_native(
        &settings,
        &conn,
        DictionarySearchArgs {
          language: Some("en".to_string()),
          query: "anchors".to_string(),
          limit: Some(5),
          mode: Some("exact".to_string()),
        },
        &None,
      )
      .expect("exact alias search");
    assert_eq!(alias_hit.total, 1);
    assert_eq!(
      alias_hit.hits[0].id,
      Some(1)
    );

    let fuzzy =
      dictionary_search_native(
        &settings,
        &conn,
        DictionarySearchArgs {
          language: Some("en".to_string()),
          query: "secure a vessel"
            .to_string(),
          limit: Some(5),
          mode: Some("fuzzy".to_string()),
        },
        &None,
      )
      .expect("fuzzy search");
    assert!(
      fuzzy
        .hits
        .iter()
        .any(|hit| hit.id == Some(1))
    );
    let morphology =
      dictionary_search_native(
        &settings,
        &conn,
        DictionarySearchArgs {
          language: Some("en".to_string()),
          query: "anchoring".to_string(),
          limit: Some(5),
          mode: Some("prefix".to_string()),
        },
        &None,
      )
      .expect(
        "morphology fallback search"
      );
    assert!(
      morphology
        .hits
        .iter()
        .any(|hit| hit.id == Some(1))
    );
    assert!(
      morphology
        .warnings
        .iter()
        .any(|warning| {
          warning.contains(
            "morphology fallback"
          )
        })
    );

    drop(conn);
    let _ = fs::remove_file(path);
  }
}
