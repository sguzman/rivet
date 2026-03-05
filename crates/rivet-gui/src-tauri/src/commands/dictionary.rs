use std::time::Instant;

use postgres::{Client, NoTls};
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use rivet_gui_shared::{
  DictionaryEntry,
  DictionaryEntryArgs,
  DictionaryMeta,
  DictionaryPronunciation,
  DictionarySearchArgs,
  DictionarySearchHit,
  DictionarySearchResult,
  DictionarySense,
};

const DEFAULT_SEARCH_LIMIT: u32 = 100;
const MAX_SEARCH_LIMIT: u32 = 500;
const SLOW_QUERY_WARN_MS: u128 = 250;

const DEFAULT_PG_HOST: &str = "127.0.0.1";
const DEFAULT_PG_PORT: u16 = 5432;
const DEFAULT_PG_USER: &str = "admin";
const DEFAULT_PG_PASSWORD: &str = "admin";
const DEFAULT_PG_DATABASE: &str = "data";
const DEFAULT_PG_SCHEMA: &str = "dictionary";
const DEFAULT_PG_SSLMODE: &str = "disable";
const DEFAULT_PG_CONNECT_TIMEOUT_SECS: u64 = 10;
const DEFAULT_PG_MAX_CONNECTION_RETRIES: u32 = 5;
const DEFAULT_PG_RETRY_BACKOFF_MS: u64 = 750;
const DEFAULT_POOL_MAX_SIZE: u32 = 8;

#[derive(Debug, serde::Deserialize, Clone)]
struct DictionaryPostgresConfig {
  host: Option<String>,
  port: Option<u16>,
  user: Option<String>,
  password: Option<String>,
  database: Option<String>,
  schema: Option<String>,
  sslmode: Option<String>,
  connect_timeout_secs: Option<u64>,
  max_connection_retries: Option<u32>,
  retry_backoff_ms: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
struct DictionaryConfig {
  enabled: Option<bool>,
  default_language: Option<String>,
  max_results: Option<u32>,
  search_mode: Option<String>,
  postgres: Option<DictionaryPostgresConfig>,
}

#[derive(Debug, Clone)]
struct DictionaryPostgresSettings {
  host: String,
  port: u16,
  user: String,
  password: String,
  database: String,
  schema: String,
  sslmode: String,
  connect_timeout_secs: u64,
  max_connection_retries: u32,
  retry_backoff_ms: u64,
}

#[derive(Debug, Clone)]
struct DictionarySettings {
  enabled: bool,
  default_language: Option<String>,
  max_results: u32,
  search_mode: String,
  postgres: DictionaryPostgresSettings,
}

#[derive(Debug, Clone)]
struct NativeHit {
  page_id: i64,
  word: String,
  language: String,
  summary: Option<String>,
}

#[derive(Debug, Clone)]
struct RelationBuckets {
  pronunciation: Vec<String>,
  part_of_speech: Vec<String>,
  etymology: Vec<String>,
  examples: Vec<String>,
  notes: Vec<String>,
  metadata: Vec<DictionaryMeta>,
}

type PgManager = PostgresConnectionManager<NoTls>;
type PgPool = Pool<PgManager>;

#[derive(Clone)]
struct CachedPool {
  key: String,
  pool: PgPool,
}

static DICTIONARY_POOL_CACHE: std::sync::OnceLock<std::sync::Mutex<Option<CachedPool>>> =
  std::sync::OnceLock::new();

fn pool_cache() -> &'static std::sync::Mutex<Option<CachedPool>> {
  DICTIONARY_POOL_CACHE.get_or_init(|| std::sync::Mutex::new(None))
}

fn normalize_search_mode(value: Option<String>) -> String {
  let Some(raw) = value else {
    return "prefix".to_string();
  };
  let mode = raw.trim().to_ascii_lowercase();
  if mode == "exact" || mode == "prefix" || mode == "fuzzy" || mode == "fts" {
    mode
  } else {
    "prefix".to_string()
  }
}

fn normalized_language(value: Option<String>) -> Option<String> {
  value.and_then(|raw| {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "*" || trimmed.eq_ignore_ascii_case("all") {
      None
    } else {
      Some(trimmed.to_string())
    }
  })
}

fn normalize_for_search(value: &str) -> String {
  value.trim().to_ascii_lowercase()
}

fn normalize_sslmode(value: Option<String>) -> String {
  let mode = value
    .unwrap_or_else(|| DEFAULT_PG_SSLMODE.to_string())
    .trim()
    .to_ascii_lowercase();
  if mode == "disable" {
    mode
  } else {
    "disable".to_string()
  }
}

fn validate_identifier(value: &str, label: &str) -> anyhow::Result<()> {
  if value.is_empty() {
    anyhow::bail!("dictionary postgres {label} cannot be empty");
  }
  let valid = value
    .chars()
    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_');
  if !valid {
    anyhow::bail!(
      "dictionary postgres {label} must match [A-Za-z0-9_]+"
    );
  }
  Ok(())
}

fn quote_ident(ident: &str) -> String {
  format!("\"{}\"", ident.replace('"', "\"\""))
}

fn map_iso_language(token: &str) -> Option<&'static str> {
  match token.trim().to_ascii_lowercase().as_str() {
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

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
  let mut out = Vec::<String>::new();
  for value in values {
    let trimmed = value.trim();
    if trimmed.is_empty() {
      continue;
    }
    if out.iter().any(|existing| existing.eq_ignore_ascii_case(trimmed)) {
      continue;
    }
    out.push(trimmed.to_string());
  }
  out
}

fn lemma_candidates(query: &str) -> Vec<String> {
  let token = query.trim().to_ascii_lowercase();
  if token.len() < 4 {
    return vec![];
  }
  let mut out = Vec::<String>::new();
  if token.ends_with("ies") && token.len() > 4 {
    out.push(format!("{}y", &token[..token.len() - 3]));
  }
  if token.ends_with("es") && token.len() > 4 {
    out.push(token[..token.len() - 2].to_string());
  }
  if token.ends_with('s') && token.len() > 3 {
    out.push(token[..token.len() - 1].to_string());
  }
  if token.ends_with("ing") && token.len() > 5 {
    let stem = token[..token.len() - 3].to_string();
    out.push(stem.clone());
    if let Some(last) = stem.chars().last() {
      out.push(format!("{stem}{last}"));
    }
    out.push(format!("{stem}e"));
  }
  if token.ends_with("ed") && token.len() > 4 {
    let stem = token[..token.len() - 2].to_string();
    out.push(stem.clone());
    out.push(format!("{stem}e"));
  }
  out.retain(|candidate| !candidate.is_empty() && candidate != &token);
  dedupe_strings(out)
}

fn classify_relation_type(relation_type: &str, target: &str, buckets: &mut RelationBuckets) {
  let relation = relation_type.trim().to_ascii_lowercase();
  let term = target.trim();
  if term.is_empty() {
    return;
  }

  if relation.contains("pron") || relation.contains("ipa") || relation.contains("phon") {
    buckets.pronunciation.push(term.to_string());
    buckets.metadata.push(DictionaryMeta {
      relation_type: relation_type.to_string(),
      target: term.to_string(),
    });
    return;
  }

  if relation == "pos"
    || relation.contains("part_of_speech")
    || relation.contains("word_class")
    || relation.contains("grammatical")
  {
    buckets.part_of_speech.push(term.to_string());
    buckets.metadata.push(DictionaryMeta {
      relation_type: relation_type.to_string(),
      target: term.to_string(),
    });
    return;
  }

  if relation.contains("etym") || relation.contains("origin") {
    buckets.etymology.push(term.to_string());
    buckets.metadata.push(DictionaryMeta {
      relation_type: relation_type.to_string(),
      target: term.to_string(),
    });
    return;
  }

  if relation.contains("example") || relation.contains("usage") {
    buckets.examples.push(term.to_string());
    buckets.metadata.push(DictionaryMeta {
      relation_type: relation_type.to_string(),
      target: term.to_string(),
    });
    return;
  }

  buckets.notes.push(format!("{relation_type}: {term}"));
  buckets.metadata.push(DictionaryMeta {
    relation_type: relation_type.to_string(),
    target: term.to_string(),
  });
}

fn relation_buckets(
  client: &mut Client,
  schema: &str,
  page_id: i64,
  language: &str,
) -> anyhow::Result<RelationBuckets> {
  let sql = format!(
    "SELECT relation_type, target_term FROM {}.relations WHERE page_id = $1 AND language = $2 ORDER BY rel_order ASC, id ASC LIMIT 512",
    quote_ident(schema),
  );
  let rows = client.query(&sql, &[&page_id, &language])?;
  let mut buckets = RelationBuckets {
    pronunciation: vec![],
    part_of_speech: vec![],
    etymology: vec![],
    examples: vec![],
    notes: vec![],
    metadata: vec![],
  };

  for row in rows {
    let relation_type: String = row.get(0);
    let target: String = row.get(1);
    classify_relation_type(&relation_type, &target, &mut buckets);
  }

  buckets.pronunciation = dedupe_strings(buckets.pronunciation);
  buckets.part_of_speech = dedupe_strings(buckets.part_of_speech);
  buckets.etymology = dedupe_strings(buckets.etymology);
  buckets.examples = dedupe_strings(buckets.examples);
  buckets.notes = dedupe_strings(buckets.notes);
  Ok(buckets)
}

fn resolve_dictionary_settings() -> anyhow::Result<DictionarySettings> {
  let config_path = resolve_config_path("rivet.toml");
  let mut enabled = true;
  let mut default_language = None;
  let mut max_results = DEFAULT_SEARCH_LIMIT;
  let mut search_mode = "prefix".to_string();
  let mut postgres = DictionaryPostgresSettings {
    host: DEFAULT_PG_HOST.to_string(),
    port: DEFAULT_PG_PORT,
    user: DEFAULT_PG_USER.to_string(),
    password: DEFAULT_PG_PASSWORD.to_string(),
    database: DEFAULT_PG_DATABASE.to_string(),
    schema: DEFAULT_PG_SCHEMA.to_string(),
    sslmode: DEFAULT_PG_SSLMODE.to_string(),
    connect_timeout_secs: DEFAULT_PG_CONNECT_TIMEOUT_SECS,
    max_connection_retries: DEFAULT_PG_MAX_CONNECTION_RETRIES,
    retry_backoff_ms: DEFAULT_PG_RETRY_BACKOFF_MS,
  };

  if config_path.is_file() {
    let raw = std::fs::read_to_string(&config_path)
      .with_context(|| format!("failed to read {}", config_path.display()))?;
    let value = toml::from_str::<toml::Value>(&raw)
      .with_context(|| format!("failed to parse {}", config_path.display()))?;
    if let Some(dictionary) = value
      .get("dictionary")
      .and_then(|entry| entry.clone().try_into::<DictionaryConfig>().ok())
    {
      if let Some(flag) = dictionary.enabled {
        enabled = flag;
      }
      default_language = normalized_language(dictionary.default_language);
      if let Some(limit) = dictionary.max_results {
        max_results = limit.clamp(1, MAX_SEARCH_LIMIT);
      }
      search_mode = normalize_search_mode(dictionary.search_mode);
      if let Some(pg) = dictionary.postgres {
        if let Some(host) = pg.host {
          postgres.host = host;
        }
        if let Some(port) = pg.port {
          postgres.port = port;
        }
        if let Some(user) = pg.user {
          postgres.user = user;
        }
        if let Some(password) = pg.password {
          postgres.password = password;
        }
        if let Some(database) = pg.database {
          postgres.database = database;
        }
        if let Some(schema) = pg.schema {
          postgres.schema = schema;
        }
        postgres.sslmode = normalize_sslmode(pg.sslmode);
        if let Some(timeout) = pg.connect_timeout_secs {
          postgres.connect_timeout_secs = timeout.max(1);
        }
        if let Some(retries) = pg.max_connection_retries {
          postgres.max_connection_retries = retries.max(1);
        }
        if let Some(backoff_ms) = pg.retry_backoff_ms {
          postgres.retry_backoff_ms = backoff_ms.max(1);
        }
      }
    }
  }

  if let Ok(env) = std::env::var("RIVET_DICTIONARY_PGHOST") {
    if !env.trim().is_empty() {
      postgres.host = env.trim().to_string();
    }
  }
  if let Ok(env) = std::env::var("RIVET_DICTIONARY_PGPORT") {
    if let Ok(port) = env.trim().parse::<u16>() {
      postgres.port = port;
    }
  }
  if let Ok(env) = std::env::var("RIVET_DICTIONARY_PGUSER") {
    if !env.trim().is_empty() {
      postgres.user = env.trim().to_string();
    }
  }
  if let Ok(env) = std::env::var("RIVET_DICTIONARY_PGPASSWORD") {
    postgres.password = env;
  }
  if let Ok(env) = std::env::var("RIVET_DICTIONARY_PGDATABASE") {
    if !env.trim().is_empty() {
      postgres.database = env.trim().to_string();
    }
  }
  if let Ok(env) = std::env::var("RIVET_DICTIONARY_PGSCHEMA") {
    if !env.trim().is_empty() {
      postgres.schema = env.trim().to_string();
    }
  }
  if let Ok(env) = std::env::var("RIVET_DICTIONARY_PGSSLMODE") {
    postgres.sslmode = normalize_sslmode(Some(env));
  }

  validate_identifier(&postgres.database, "database")?;
  validate_identifier(&postgres.schema, "schema")?;

  Ok(DictionarySettings {
    enabled,
    default_language,
    max_results,
    search_mode,
    postgres,
  })
}

fn settings_key(settings: &DictionarySettings) -> String {
  format!(
    "{}:{}:{}:{}:{}:{}",
    settings.postgres.host,
    settings.postgres.port,
    settings.postgres.user,
    settings.postgres.database,
    settings.postgres.schema,
    settings.postgres.sslmode,
  )
}

fn build_pg_config(pg: &DictionaryPostgresSettings) -> anyhow::Result<postgres::Config> {
  let mut cfg = postgres::Config::new();
  cfg.host(&pg.host);
  cfg.port(pg.port);
  cfg.user(&pg.user);
  cfg.password(&pg.password);
  cfg.dbname(&pg.database);
  cfg.connect_timeout(std::time::Duration::from_secs(pg.connect_timeout_secs));
  if pg.sslmode != "disable" {
    anyhow::bail!("unsupported dictionary.postgres.sslmode {}; only disable is supported", pg.sslmode);
  }
  Ok(cfg)
}

fn ensure_required_schema(client: &mut Client, schema: &str) -> anyhow::Result<()> {
  let schema_exists = client
    .query_one(
      "SELECT EXISTS(SELECT 1 FROM information_schema.schemata WHERE schema_name = $1)",
      &[&schema],
    )?
    .get::<_, bool>(0);
  if !schema_exists {
    anyhow::bail!("dictionary schema missing: {schema}");
  }

  let required_tables = ["pages", "definitions", "relations", "lemma_aliases"];
  for table in required_tables {
    let exists = client
      .query_one(
        "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_schema = $1 AND table_name = $2)",
        &[&schema, &table],
      )?
      .get::<_, bool>(0);
    if !exists {
      anyhow::bail!("dictionary schema mismatch: missing table `{table}`");
    }
  }

  let required_columns = [
    ("pages", "id"),
    ("pages", "title"),
    ("definitions", "id"),
    ("definitions", "page_id"),
    ("definitions", "language"),
    ("definitions", "def_order"),
    ("definitions", "definition_text"),
    ("relations", "page_id"),
    ("relations", "language"),
    ("relations", "relation_type"),
    ("relations", "rel_order"),
    ("relations", "target_term"),
    ("lemma_aliases", "page_id"),
    ("lemma_aliases", "alias"),
    ("lemma_aliases", "language"),
  ];

  for (table, column) in required_columns {
    let exists = client
      .query_one(
        "SELECT EXISTS(
           SELECT 1
           FROM information_schema.columns
           WHERE table_schema = $1
             AND table_name = $2
             AND column_name = $3
         )",
        &[&schema, &table, &column],
      )?
      .get::<_, bool>(0);
    if !exists {
      anyhow::bail!("dictionary schema mismatch: missing {table}.{column}");
    }
  }

  Ok(())
}

fn create_pool(settings: &DictionarySettings) -> anyhow::Result<PgPool> {
  let manager = PostgresConnectionManager::new(build_pg_config(&settings.postgres)?, NoTls);
  let pool = Pool::builder()
    .max_size(DEFAULT_POOL_MAX_SIZE)
    .connection_timeout(std::time::Duration::from_secs(
      settings.postgres.connect_timeout_secs,
    ))
    .build(manager)
    .context("failed to create dictionary postgres pool")?;

  let retries = settings.postgres.max_connection_retries.max(1);
  for attempt in 1..=retries {
    match pool.get() {
      Ok(mut conn) => {
        ensure_required_schema(&mut conn, &settings.postgres.schema)?;
        return Ok(pool);
      }
      Err(error) => {
        if attempt == retries {
          return Err(anyhow::Error::new(error))
            .context("failed to acquire dictionary postgres connection");
        }
        tracing::warn!(
          attempt,
          retries,
          error = %error,
          "retrying dictionary postgres connection"
        );
        std::thread::sleep(std::time::Duration::from_millis(
          settings.postgres.retry_backoff_ms,
        ));
      }
    }
  }

  anyhow::bail!("exhausted dictionary postgres connection retries")
}

fn dictionary_pool(settings: &DictionarySettings) -> anyhow::Result<PgPool> {
  let key = settings_key(settings);
  let cache = pool_cache();
  let mut guard = cache
    .lock()
    .map_err(|_| anyhow::anyhow!("dictionary pool cache mutex poisoned"))?;
  if let Some(entry) = guard.as_ref() {
    if entry.key == key {
      return Ok(entry.pool.clone());
    }
  }

  let pool = create_pool(settings)?;
  *guard = Some(CachedPool { key, pool: pool.clone() });
  Ok(pool)
}

fn ensure_dictionary_ready() -> anyhow::Result<(DictionarySettings, PgPool)> {
  let settings = resolve_dictionary_settings()?;
  if !settings.enabled {
    anyhow::bail!("dictionary is disabled in config ([dictionary].enabled = false)");
  }
  let pool = dictionary_pool(&settings)?;
  Ok((settings, pool))
}

async fn run_dictionary_blocking<T, F>(operation: &'static str, task: F) -> anyhow::Result<T>
where
  T: Send + 'static,
  F: FnOnce() -> anyhow::Result<T> + Send + 'static,
{
  tokio::task::spawn_blocking(task)
    .await
    .map_err(|join_err| anyhow::anyhow!("dictionary {operation} blocking task failed: {join_err}"))?
}

fn log_if_slow(label: &str, started: Instant, request_id: &Option<String>, detail: &str) {
  let elapsed_ms = started.elapsed().as_millis();
  if elapsed_ms >= SLOW_QUERY_WARN_MS {
    tracing::warn!(request_id = ?request_id, elapsed_ms, detail, "slow dictionary query: {label}");
  }
}

fn language_exists(client: &mut Client, schema: &str, language: &str) -> anyhow::Result<bool> {
  let sql = format!(
    "SELECT EXISTS(SELECT 1 FROM {}.definitions WHERE LOWER(language) = LOWER($1) LIMIT 1)",
    quote_ident(schema),
  );
  let exists = client.query_one(&sql, &[&language])?.get::<_, bool>(0);
  Ok(exists)
}

fn resolve_language_filter(
  client: &mut Client,
  schema: &str,
  language: Option<String>,
) -> anyhow::Result<(Option<String>, Vec<String>)> {
  let Some(token) = language else {
    return Ok((None, vec![]));
  };
  if language_exists(client, schema, &token)? {
    return Ok((Some(token), vec![]));
  }

  let mut warnings = Vec::<String>::new();
  if let Some(mapped) = map_iso_language(&token)
    && language_exists(client, schema, mapped)?
  {
    warnings.push(format!("language filter `{token}` mapped to `{mapped}`"));
    return Ok((Some(mapped.to_string()), warnings));
  }

  warnings.push(format!(
    "language filter `{token}` has no matches; falling back to all languages"
  ));
  Ok((None, warnings))
}

fn detect_fts_table(client: &mut Client, schema: &str) -> anyhow::Result<Option<String>> {
  let sql = "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_name = ANY($2) ORDER BY table_name";
  let candidates = vec!["page_fts"];
  let rows = client.query(sql, &[&schema, &candidates])?;
  Ok(rows.first().map(|row| row.get::<_, String>(0)))
}

fn run_search_hits(
  client: &mut Client,
  schema: &str,
  mode: &str,
  query: &str,
  language: &Option<String>,
  limit: u32,
) -> anyhow::Result<Vec<NativeHit>> {
  let sql = format!(
    "WITH title_hits AS (
       SELECT p.id AS page_id,
              p.title AS word,
              d.language AS language,
              (
                SELECT d2.definition_text
                FROM {schema}.definitions d2
                WHERE d2.page_id = p.id
                  AND d2.language = d.language
                ORDER BY d2.def_order ASC
                LIMIT 1
              ) AS summary,
              0 AS source_rank
       FROM {schema}.pages p
       JOIN {schema}.definitions d ON d.page_id = p.id
       WHERE (
              ($1 = 'exact' AND LOWER(p.title) = LOWER($2))
           OR ($1 = 'prefix' AND p.title ILIKE $3)
           OR ($1 = 'fuzzy' AND (
                 p.title ILIKE $4 OR EXISTS (
                   SELECT 1
                   FROM {schema}.definitions d3
                   WHERE d3.page_id = p.id
                     AND d3.language = d.language
                     AND COALESCE(d3.normalized_text, d3.definition_text) ILIKE $4
                 )
               ))
             )
         AND ($5::TEXT IS NULL OR LOWER(d.language) = LOWER($5::TEXT))
       GROUP BY p.id, p.title, d.language
     ),
     alias_hits AS (
       SELECT la.page_id AS page_id,
              la.alias AS word,
              COALESCE(la.language, d.language) AS language,
              (
                SELECT d2.definition_text
                FROM {schema}.definitions d2
                WHERE d2.page_id = la.page_id
                  AND d2.language = COALESCE(la.language, d.language)
                ORDER BY d2.def_order ASC
                LIMIT 1
              ) AS summary,
              1 AS source_rank
       FROM {schema}.lemma_aliases la
       JOIN {schema}.definitions d ON d.page_id = la.page_id
       WHERE (
              ($1 = 'exact' AND LOWER(la.alias) = LOWER($2))
           OR ($1 = 'prefix' AND la.alias ILIKE $3)
           OR ($1 = 'fuzzy' AND (
                 la.alias ILIKE $4 OR COALESCE(la.normalized_alias, la.alias) ILIKE $4
               ))
             )
         AND ($5::TEXT IS NULL OR LOWER(COALESCE(la.language, d.language)) = LOWER($5::TEXT))
       GROUP BY la.page_id, la.alias, COALESCE(la.language, d.language)
     ),
     merged AS (
       SELECT page_id, word, language, summary, source_rank FROM title_hits
       UNION
       SELECT page_id, word, language, summary, source_rank FROM alias_hits
     )
     SELECT page_id, word, language, summary
     FROM merged
     ORDER BY source_rank ASC, word ASC
     LIMIT $6",
    schema = quote_ident(schema),
  );

  let exact = query.to_string();
  let prefix = format!("{query}%");
  let fuzzy = format!("%{query}%");
  let limit_i64 = i64::from(limit);
  let rows = client.query(&sql, &[&mode, &exact, &prefix, &fuzzy, language, &limit_i64])?;
  let mut out = Vec::<NativeHit>::new();
  for row in rows {
    out.push(NativeHit {
      page_id: row.get(0),
      word: row.get(1),
      language: row.get(2),
      summary: row.get(3),
    });
  }
  Ok(out)
}

fn run_search_count(
  client: &mut Client,
  schema: &str,
  mode: &str,
  query: &str,
  language: &Option<String>,
) -> anyhow::Result<i64> {
  let sql = format!(
    "WITH title_hits AS (
       SELECT p.id AS page_id,
              p.title AS word,
              d.language AS language
       FROM {schema}.pages p
       JOIN {schema}.definitions d ON d.page_id = p.id
       WHERE (
              ($1 = 'exact' AND LOWER(p.title) = LOWER($2))
           OR ($1 = 'prefix' AND p.title ILIKE $3)
           OR ($1 = 'fuzzy' AND (
                 p.title ILIKE $4 OR EXISTS (
                   SELECT 1
                   FROM {schema}.definitions d3
                   WHERE d3.page_id = p.id
                     AND d3.language = d.language
                     AND COALESCE(d3.normalized_text, d3.definition_text) ILIKE $4
                 )
               ))
             )
         AND ($5::TEXT IS NULL OR LOWER(d.language) = LOWER($5::TEXT))
       GROUP BY p.id, p.title, d.language
     ),
     alias_hits AS (
       SELECT la.page_id AS page_id,
              la.alias AS word,
              COALESCE(la.language, d.language) AS language
       FROM {schema}.lemma_aliases la
       JOIN {schema}.definitions d ON d.page_id = la.page_id
       WHERE (
              ($1 = 'exact' AND LOWER(la.alias) = LOWER($2))
           OR ($1 = 'prefix' AND la.alias ILIKE $3)
           OR ($1 = 'fuzzy' AND (
                 la.alias ILIKE $4 OR COALESCE(la.normalized_alias, la.alias) ILIKE $4
               ))
             )
         AND ($5::TEXT IS NULL OR LOWER(COALESCE(la.language, d.language)) = LOWER($5::TEXT))
       GROUP BY la.page_id, la.alias, COALESCE(la.language, d.language)
     ),
     merged AS (
       SELECT page_id, word, language FROM title_hits
       UNION
       SELECT page_id, word, language FROM alias_hits
     )
     SELECT COUNT(1) FROM merged",
    schema = quote_ident(schema),
  );

  let exact = query.to_string();
  let prefix = format!("{query}%");
  let fuzzy = format!("%{query}%");
  let total = client.query_one(&sql, &[&mode, &exact, &prefix, &fuzzy, language])?.get::<_, i64>(0);
  Ok(total)
}

fn run_fts_hits(
  client: &mut Client,
  schema: &str,
  table: &str,
  query: &str,
  language: &Option<String>,
  limit: u32,
) -> anyhow::Result<Vec<NativeHit>> {
  let sql = if table == "page_fts" {
    format!(
      "SELECT p.id, p.title, d.language,
              (
                SELECT d2.definition_text
                FROM {schema}.definitions d2
                WHERE d2.page_id = p.id
                  AND d2.language = d.language
                ORDER BY d2.def_order ASC
                LIMIT 1
              ) AS summary
       FROM {schema}.page_fts f
       JOIN {schema}.pages p ON p.id = f.page_id
       JOIN {schema}.definitions d ON d.page_id = p.id
       WHERE to_tsvector('simple', COALESCE(f.title, '') || ' ' || COALESCE(f.plain_text, '')) @@ plainto_tsquery('simple', $1)
         AND ($2::TEXT IS NULL OR LOWER(d.language) = LOWER($2::TEXT))
       GROUP BY p.id, p.title, d.language
       ORDER BY p.title ASC
       LIMIT $3",
      schema = quote_ident(schema)
    )
  } else {
    anyhow::bail!("unsupported fts table {table}");
  };

  let limit_i64 = i64::from(limit);
  let rows = client.query(&sql, &[&query, language, &limit_i64])?;
  let mut out = Vec::<NativeHit>::new();
  for row in rows {
    out.push(NativeHit {
      page_id: row.get(0),
      word: row.get(1),
      language: row.get(2),
      summary: row.get(3),
    });
  }
  Ok(out)
}

fn dictionary_languages_native(settings: &DictionarySettings, client: &mut Client) -> anyhow::Result<Vec<String>> {
  let sql = format!(
    "SELECT DISTINCT language FROM {}.definitions WHERE TRIM(language) <> '' ORDER BY language ASC LIMIT 4096",
    quote_ident(&settings.postgres.schema),
  );
  let rows = client.query(&sql, &[])?;
  let mut out = rows
    .into_iter()
    .map(|row| row.get::<_, String>(0))
    .filter(|value| !value.trim().is_empty())
    .collect::<Vec<_>>();
  if out.is_empty()
    && let Some(default_language) = settings.default_language.as_ref()
  {
    out.push(default_language.clone());
  }
  Ok(out)
}

fn dictionary_search_native(
  settings: &DictionarySettings,
  client: &mut Client,
  args: DictionarySearchArgs,
  request_id: &Option<String>,
) -> anyhow::Result<DictionarySearchResult> {
  let query = args.query.trim();
  if query.is_empty() {
    return Ok(DictionarySearchResult {
      query: query.to_string(),
      language: normalized_language(args.language),
      hits: vec![],
      total: 0,
      truncated: false,
      warnings: vec!["query is empty".to_string()],
    });
  }

  let requested_language = normalized_language(args.language).or(settings.default_language.clone());
  let (language, mut warnings) = resolve_language_filter(client, &settings.postgres.schema, requested_language)?;

  let requested_mode = normalize_search_mode(args.mode);
  let effective_mode = if requested_mode.trim().is_empty() {
    settings.search_mode.clone()
  } else {
    requested_mode
  };
  let limit = args.limit.unwrap_or(settings.max_results).clamp(1, MAX_SEARCH_LIMIT);

  let list_started = Instant::now();
  let mut rows = if effective_mode == "fts" {
    match detect_fts_table(client, &settings.postgres.schema)? {
      Some(table) => match run_fts_hits(client, &settings.postgres.schema, &table, query, &language, limit) {
        Ok(payload) => payload,
        Err(err) => {
          warnings.push(format!("fts query unavailable ({err}); falling back to fuzzy search"));
          run_search_hits(client, &settings.postgres.schema, "fuzzy", query, &language, limit)?
        }
      },
      None => {
        warnings.push("fts mode requested but no page_fts table found; falling back to fuzzy search".to_string());
        run_search_hits(client, &settings.postgres.schema, "fuzzy", query, &language, limit)?
      }
    }
  } else {
    run_search_hits(client, &settings.postgres.schema, &effective_mode, query, &language, limit)?
  };
  log_if_slow("search.list", list_started, request_id, "dictionary search list query");

  let count_started = Instant::now();
  let mut total = if effective_mode == "fts" {
    rows.len() as i64
  } else {
    run_search_count(client, &settings.postgres.schema, &effective_mode, query, &language)?
  };
  log_if_slow("search.count", count_started, request_id, "dictionary search count query");

  if rows.is_empty() {
    let candidates = lemma_candidates(query);
    if !candidates.is_empty() {
      let fallback_started = Instant::now();
      let mut fallback_rows = Vec::<NativeHit>::new();
      for candidate in &candidates {
        let candidate_hits = run_search_hits(client, &settings.postgres.schema, "exact", candidate, &language, limit)?;
        for hit in candidate_hits {
          let exists = fallback_rows
            .iter()
            .any(|entry| entry.page_id == hit.page_id && entry.language.eq_ignore_ascii_case(&hit.language));
          if exists {
            continue;
          }
          fallback_rows.push(hit);
          if fallback_rows.len() >= limit as usize {
            break;
          }
        }
        if fallback_rows.len() >= limit as usize {
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
        "dictionary morphology fallback",
      );
    }
  }

  let enrich_started = Instant::now();
  let mut hits = Vec::<DictionarySearchHit>::new();
  for row in rows {
    let relation = relation_buckets(client, &settings.postgres.schema, row.page_id, &row.language)?;
    hits.push(DictionarySearchHit {
      id: Some(row.page_id),
      word: row.word,
      language: Some(row.language),
      part_of_speech: relation.part_of_speech.first().cloned(),
      pronunciation: relation.pronunciation.first().cloned(),
      summary: row.summary,
      source_table: "postgres.pages/definitions/lemma_aliases".to_string(),
      matched_by_prefix: effective_mode == "prefix",
    });
  }
  log_if_slow("search.enrich", enrich_started, request_id, "dictionary hit enrichment");

  Ok(DictionarySearchResult {
    query: query.to_string(),
    language,
    hits,
    total: total.max(0) as u64,
    truncated: total.max(0) as u64 > limit as u64,
    warnings,
  })
}

fn dictionary_entry_native(
  settings: &DictionarySettings,
  client: &mut Client,
  args: DictionaryEntryArgs,
  request_id: &Option<String>,
) -> anyhow::Result<Option<DictionaryEntry>> {
  let requested_language = normalized_language(args.language).or(settings.default_language.clone());
  let (requested_language, _warnings) = resolve_language_filter(client, &settings.postgres.schema, requested_language)?;
  let requested_word = args.word.and_then(|word| {
    let trimmed = word.trim();
    if trimmed.is_empty() {
      None
    } else {
      Some(trimmed.to_string())
    }
  });

  if args.id.is_none() && requested_word.is_none() {
    anyhow::bail!("dictionary_entry requires id or word");
  }

  let schema = quote_ident(&settings.postgres.schema);

  let resolve_started = Instant::now();
  let resolved: Option<(i64, String)> = if let Some(page_id) = args.id {
    if let Some(language) = requested_language.as_ref() {
      Some((page_id, language.clone()))
    } else {
      let sql = format!(
        "SELECT language FROM {schema}.definitions WHERE page_id = $1 GROUP BY language ORDER BY COUNT(*) DESC, language ASC LIMIT 1"
      );
      client
        .query_opt(&sql, &[&page_id])?
        .map(|row| (page_id, row.get::<_, String>(0)))
    }
  } else {
    let word = requested_word.clone().unwrap_or_default();
    let sql = format!(
      "WITH title_match AS (
         SELECT p.id AS page_id,
                d.language AS language,
                0 AS rank
         FROM {schema}.pages p
         JOIN {schema}.definitions d ON d.page_id = p.id
         WHERE LOWER(p.title) = LOWER($1)
           AND ($2::TEXT IS NULL OR LOWER(d.language) = LOWER($3::TEXT))
         GROUP BY p.id, d.language
       ),
       alias_match AS (
         SELECT la.page_id AS page_id,
                COALESCE(la.language, d.language) AS language,
                1 AS rank
         FROM {schema}.lemma_aliases la
         JOIN {schema}.definitions d ON d.page_id = la.page_id
         WHERE LOWER(la.alias) = LOWER($4)
           AND ($5::TEXT IS NULL OR LOWER(COALESCE(la.language, d.language)) = LOWER($6::TEXT))
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
       LIMIT 1"
    );
    client.query_opt(
      &sql,
      &[&word, &requested_language, &requested_language, &word, &requested_language, &requested_language],
    )?
    .map(|row| (row.get::<_, i64>(0), row.get::<_, String>(1)))
  };
  log_if_slow("entry.resolve", resolve_started, request_id, "resolve entry target");

  let Some((page_id, language)) = resolved else {
    return Ok(None);
  };

  let word_started = Instant::now();
  let sql_word = format!("SELECT title FROM {schema}.pages WHERE id = $1 LIMIT 1");
  let word: String = client.query_one(&sql_word, &[&page_id])?.get(0);
  log_if_slow("entry.word", word_started, request_id, "fetch page title");

  let defs_started = Instant::now();
  let sql_defs = format!(
    "SELECT definition_text FROM {schema}.definitions WHERE page_id = $1 AND language = $2 ORDER BY def_order ASC"
  );
  let definitions_rows = client.query(&sql_defs, &[&page_id, &language])?;
  let definitions = definitions_rows.into_iter().map(|row| row.get::<_, String>(0)).collect::<Vec<_>>();
  log_if_slow("entry.definitions", defs_started, request_id, "fetch entry definitions");

  let rel_started = Instant::now();
  let relation = relation_buckets(client, &settings.postgres.schema, page_id, &language)?;
  log_if_slow("entry.relations", rel_started, request_id, "fetch entry relations");

  let senses = definitions
    .iter()
    .enumerate()
    .map(|(idx, text)| DictionarySense { order: idx as u32 + 1, text: text.clone() })
    .collect::<Vec<_>>();
  let pronunciations = relation
    .pronunciation
    .iter()
    .map(|text| DictionaryPronunciation {
      text: text.clone(),
      system: None,
    })
    .collect::<Vec<_>>();

  Ok(Some(DictionaryEntry {
    id: Some(page_id),
    word,
    language: Some(language),
    part_of_speech: relation.part_of_speech.first().cloned(),
    pronunciation: relation.pronunciation.first().cloned(),
    etymology: if relation.etymology.is_empty() {
      None
    } else {
      Some(relation.etymology.join(" | "))
    },
    definitions,
    senses,
    pronunciations,
    examples: relation.examples,
    notes: relation.notes,
    metadata: relation.metadata,
    source_table: "postgres.pages/definitions/relations/lemma_aliases".to_string(),
  }))
}

#[tauri::command]
#[tracing::instrument(fields(request_id = ?request_id))]
pub async fn dictionary_languages(request_id: Option<String>) -> Result<Vec<String>, String> {
  let started = Instant::now();
  tracing::info!(request_id = ?request_id, "dictionary_languages command invoked");

  let result = run_dictionary_blocking("languages", move || -> anyhow::Result<Vec<String>> {
    let (settings, pool) = ensure_dictionary_ready()?;
    let mut client = pool
      .get()
      .map_err(anyhow::Error::new)
      .context("failed to checkout dictionary postgres connection")?;
    dictionary_languages_native(&settings, &mut client)
  })
  .await;

  let elapsed_ms = started.elapsed().as_millis();
  if let Err(err) = result.as_ref() {
    tracing::error!(request_id = ?request_id, elapsed_ms, error = %err, "dictionary_languages command failed");
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
  request_id: Option<String>,
) -> Result<DictionarySearchResult, String> {
  let started = Instant::now();
  let normalized_query = normalize_for_search(&args.query);
  let request_id_for_query = request_id.clone();
  tracing::info!(
    request_id = ?request_id,
    query_len = args.query.len(),
    language = ?args.language,
    "dictionary_search command invoked"
  );

  let result = run_dictionary_blocking("search", move || -> anyhow::Result<DictionarySearchResult> {
    let (settings, pool) = ensure_dictionary_ready()?;
    let mut client = pool
      .get()
      .map_err(anyhow::Error::new)
      .context("failed to checkout dictionary postgres connection")?;
    dictionary_search_native(
      &settings,
      &mut client,
      DictionarySearchArgs {
        language: args.language,
        query: normalized_query,
        limit: args.limit,
        mode: args.mode,
      },
      &request_id_for_query,
    )
  })
  .await;

  let elapsed_ms = started.elapsed().as_millis();
  if let Err(err) = result.as_ref() {
    tracing::error!(request_id = ?request_id, elapsed_ms, error = %err, "dictionary_search command failed");
  } else if let Ok(payload) = result.as_ref() {
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
  request_id: Option<String>,
) -> Result<Option<DictionaryEntry>, String> {
  let started = Instant::now();
  tracing::info!(request_id = ?request_id, id = ?args.id, language = ?args.language, "dictionary_entry command invoked");
  let request_id_for_query = request_id.clone();

  let result = run_dictionary_blocking("entry", move || -> anyhow::Result<Option<DictionaryEntry>> {
    let (settings, pool) = ensure_dictionary_ready()?;
    let mut client = pool
      .get()
      .map_err(anyhow::Error::new)
      .context("failed to checkout dictionary postgres connection")?;
    dictionary_entry_native(&settings, &mut client, args, &request_id_for_query)
  })
  .await;

  let elapsed_ms = started.elapsed().as_millis();
  if let Err(err) = result.as_ref() {
    tracing::error!(request_id = ?request_id, elapsed_ms, error = %err, "dictionary_entry command failed");
  } else {
    tracing::debug!(
      request_id = ?request_id,
      elapsed_ms,
      found = result.as_ref().ok().and_then(|entry| entry.as_ref()).is_some(),
      "dictionary_entry command completed"
    );
  }

  result.map_err(err_to_string)
}

#[cfg(test)]
mod dictionary_tests {
  use super::{
    classify_relation_type,
    dedupe_strings,
    lemma_candidates,
    map_iso_language,
    normalize_search_mode,
    normalized_language,
    quote_ident,
    RelationBuckets,
  };
  use postgres::NoTls;

  #[test]
  fn normalized_language_handles_all() {
    assert_eq!(normalized_language(Some("all".to_string())), None);
    assert_eq!(normalized_language(Some("en".to_string())), Some("en".to_string()));
  }

  #[test]
  fn dedupe_strings_is_case_insensitive() {
    let out = dedupe_strings(vec![
      "Alpha".to_string(),
      "alpha".to_string(),
      "Beta".to_string(),
      "".to_string(),
    ]);
    assert_eq!(out, vec!["Alpha".to_string(), "Beta".to_string()]);
  }

  #[test]
  fn normalize_search_mode_defaults_to_prefix() {
    assert_eq!(normalize_search_mode(None), "prefix".to_string());
    assert_eq!(normalize_search_mode(Some("nope".to_string())), "prefix".to_string());
    assert_eq!(normalize_search_mode(Some("fuzzy".to_string())), "fuzzy".to_string());
    assert_eq!(normalize_search_mode(Some("fts".to_string())), "fts".to_string());
  }

  #[test]
  fn lemma_candidates_reduces_inflections() {
    let candidates = lemma_candidates("anchoring");
    assert!(candidates.iter().any(|candidate| candidate == "anchor"));
  }

  #[test]
  fn map_iso_language_maps_en() {
    assert_eq!(map_iso_language("en"), Some("English"));
    assert_eq!(map_iso_language("xx"), None);
  }

  #[test]
  fn classify_relation_type_buckets_synonym() {
    let mut buckets = RelationBuckets {
      pronunciation: vec![],
      part_of_speech: vec![],
      etymology: vec![],
      examples: vec![],
      notes: vec![],
      metadata: vec![],
    };
    classify_relation_type("synonym", "anchor", &mut buckets);
    assert_eq!(buckets.notes.len(), 1);
    assert_eq!(buckets.metadata.len(), 1);
    assert_eq!(buckets.metadata[0].relation_type, "synonym");
    assert_eq!(buckets.metadata[0].target, "anchor");
  }

  #[test]
  fn postgres_integration_smoke_when_env_present() {
    let Ok(url) = std::env::var("RIVET_DICTIONARY_TEST_DSN") else {
      return;
    };
    let schema = std::env::var("RIVET_DICTIONARY_TEST_SCHEMA").unwrap_or_else(|_| "dictionary".to_string());
    let mut cfg = url.parse::<postgres::Config>().expect("parse postgres dsn");
    cfg.connect_timeout(std::time::Duration::from_secs(5));
    let mut client = cfg.connect(NoTls).expect("connect postgres");
    let sql = format!("SELECT COUNT(*) FROM {}.pages", quote_ident(&schema));
    let count: i64 = client.query_one(&sql, &[]).expect("query pages count").get(0);
    assert!(count >= 0);
  }
}
