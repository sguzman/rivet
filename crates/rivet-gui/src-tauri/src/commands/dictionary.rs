use std::time::Instant;

use rivet_gui_shared::{
  DictionaryEntry,
  DictionaryEntryArgs,
  DictionarySearchArgs,
  DictionarySearchHit,
  DictionarySearchResult
};
use rusqlite::types::{
  Value as SqlValue,
  ValueRef
};
use rusqlite::{
  Connection,
  OpenFlags
};

const DEFAULT_DICTIONARY_PATH: &str =
  "/win/linux/data/wiktionary/\
   wiktionary.sqlite";
const DEFAULT_SEARCH_LIMIT: u32 = 100;
const MAX_SEARCH_LIMIT: u32 = 500;

#[derive(Debug, serde::Deserialize)]
struct DictionaryConfig {
  enabled:          Option<bool>,
  sqlite_path:      Option<String>,
  default_language: Option<String>,
  max_results:      Option<u32>
}

#[derive(Debug, Clone)]
struct DictionarySettings {
  enabled:          bool,
  sqlite_path:      std::path::PathBuf,
  default_language: Option<String>,
  max_results:      u32
}

#[derive(Debug, Clone)]
struct DiscoveredSchema {
  table:             String,
  id_col:            Option<String>,
  word_col:          String,
  language_col:      Option<String>,
  part_of_speech_col: Option<String>,
  pronunciation_col: Option<String>,
  definition_col:    Option<String>,
  etymology_col:     Option<String>,
  example_col:       Option<String>,
  notes_col:         Option<String>
}

fn quote_identifier(
  ident: &str
) -> String {
  format!("\"{}\"", ident.replace('"', "\"\""))
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

fn split_text_sections(
  value: Option<String>
) -> Vec<String> {
  let Some(text) = value else {
    return vec![];
  };
  text
    .lines()
    .flat_map(|line| line.split(" | "))
    .map(str::trim)
    .filter(|entry| !entry.is_empty())
    .map(ToString::to_string)
    .take(32)
    .collect()
}

fn value_ref_to_string(
  value: ValueRef<'_>
) -> Option<String> {
  match value {
    | ValueRef::Null => None,
    | ValueRef::Text(raw) => Some(
      String::from_utf8_lossy(raw)
        .trim()
        .to_string()
    ),
    | ValueRef::Integer(v) => {
      Some(v.to_string())
    }
    | ValueRef::Real(v) => {
      Some(v.to_string())
    }
    | ValueRef::Blob(_raw) => None
  }
}

fn column_case_insensitive(
  columns: &[String],
  names: &[&str]
) -> Option<String> {
  for name in names {
    if let Some(found) = columns
      .iter()
      .find(|col| {
        col.eq_ignore_ascii_case(name)
      })
    {
      return Some(found.clone());
    }
  }
  None
}

fn open_dictionary_connection(
  path: &std::path::Path
) -> anyhow::Result<Connection> {
  let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
    | OpenFlags::SQLITE_OPEN_URI
    | OpenFlags::SQLITE_OPEN_NO_MUTEX;
  Connection::open_with_flags(path, flags)
    .with_context(|| {
      format!(
        "failed to open dictionary sqlite \
         DB at {}",
        path.display()
      )
    })
}

fn resolve_dictionary_settings(
) -> anyhow::Result<DictionarySettings> {
  let config_path =
    resolve_config_path("rivet.toml");
  let mut enabled = true;
  let mut sqlite_path =
    std::path::PathBuf::from(
      DEFAULT_DICTIONARY_PATH
    );
  let mut default_language = None;
  let mut max_results =
    DEFAULT_SEARCH_LIMIT;

  if config_path.is_file() {
    let raw = std::fs::read_to_string(
      &config_path
    )
    .with_context(|| {
      format!(
        "failed to read {}",
        config_path.display()
      )
    })?;
    let value =
      toml::from_str::<toml::Value>(
        &raw
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
        sqlite_path = std::path::PathBuf::from(
          path
        );
      }
      default_language =
        normalized_language(
          dictionary.default_language
        );
      if let Some(limit) =
        dictionary.max_results
      {
        max_results = limit.clamp(
          1,
          MAX_SEARCH_LIMIT
        );
      }
    }
  }

  if let Ok(env_path) = std::env::var(
    "RIVET_DICTIONARY_SQLITE"
  ) {
    let trimmed = env_path.trim();
    if !trimmed.is_empty() {
      sqlite_path = std::path::PathBuf::from(
        trimmed
      );
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
    max_results
  })
}

fn discover_schema(
  conn: &Connection
) -> anyhow::Result<DiscoveredSchema> {
  let mut stmt = conn.prepare(
    "SELECT name FROM sqlite_master \
     WHERE type='table' AND name NOT \
     LIKE 'sqlite_%' ORDER BY name"
  )?;
  let tables = stmt
    .query_map([], |row| row.get::<
      _,
      String,
    >(0))?
    .collect::<Result<Vec<_>, _>>()?;

  let mut best: Option<(
    i32,
    DiscoveredSchema
  )> = None;

  for table in tables {
    let pragma = format!(
      "PRAGMA table_info({})",
      quote_identifier(&table)
    );
    let mut table_stmt =
      conn.prepare(&pragma)?;
    let mut columns = table_stmt
      .query_map([], |row| {
        row.get::<_, String>(1)
      })?
      .collect::<Result<Vec<_>, _>>()?;
    columns.sort();

    let word_col = column_case_insensitive(
      &columns,
      &[
        "word",
        "lemma",
        "term",
        "title",
        "headword"
      ]
    );
    let Some(word_col) = word_col
    else {
      continue;
    };

    let id_col = column_case_insensitive(
      &columns,
      &["id", "entry_id", "rowid"]
    );
    let language_col =
      column_case_insensitive(
        &columns,
        &[
          "language",
          "lang",
          "lang_code",
          "locale"
        ]
      );
    let part_of_speech_col =
      column_case_insensitive(
        &columns,
        &[
          "part_of_speech",
          "pos",
          "word_type"
        ]
      );
    let pronunciation_col =
      column_case_insensitive(
        &columns,
        &[
          "pronunciation",
          "ipa",
          "phonetic",
          "pron"
        ]
      );
    let definition_col =
      column_case_insensitive(
        &columns,
        &[
          "definition",
          "definitions",
          "sense",
          "meaning",
          "gloss"
        ]
      );
    let etymology_col =
      column_case_insensitive(
        &columns,
        &["etymology", "origin"]
      );
    let example_col =
      column_case_insensitive(
        &columns,
        &[
          "example",
          "examples",
          "usage_example"
        ]
      );
    let notes_col = column_case_insensitive(
      &columns,
      &[
        "note",
        "notes",
        "usage_notes"
      ]
    );

    let mut score = 0;
    if language_col.is_some() {
      score += 3;
    }
    if definition_col.is_some() {
      score += 4;
    }
    if pronunciation_col.is_some() {
      score += 2;
    }
    if part_of_speech_col.is_some() {
      score += 2;
    }
    if table
      .to_ascii_lowercase()
      .contains("wik")
    {
      score += 1;
    }
    if table
      .to_ascii_lowercase()
      .contains("entry")
    {
      score += 1;
    }

    let schema = DiscoveredSchema {
      table,
      id_col,
      word_col,
      language_col,
      part_of_speech_col,
      pronunciation_col,
      definition_col,
      etymology_col,
      example_col,
      notes_col
    };

    match best.as_ref() {
      | Some((best_score, _))
        if score <= *best_score => {}
      | _ => {
        best = Some((score, schema));
      }
    }
  }

  best
    .map(|(_, schema)| schema)
    .ok_or_else(|| {
      anyhow::anyhow!(
        "could not discover dictionary \
         schema (expected columns like \
         word/lemma/term)"
      )
    })
}

fn ensure_dictionary_ready()
-> anyhow::Result<(
  DictionarySettings,
  Connection,
  DiscoveredSchema,
)> {
  let settings =
    resolve_dictionary_settings()?;
  if !settings.enabled {
    anyhow::bail!(
      "dictionary is disabled in \
       config ([dictionary].enabled \
       = false)"
    );
  }
  if !settings.sqlite_path.is_file() {
    anyhow::bail!(
      "dictionary sqlite path missing: \
       {}",
      settings.sqlite_path.display()
    );
  }
  let conn = open_dictionary_connection(
    &settings.sqlite_path
  )?;
  let schema = discover_schema(&conn)?;
  Ok((settings, conn, schema))
}

#[tauri::command]
#[tracing::instrument(fields(request_id = ?request_id))]
pub async fn dictionary_languages(
  request_id: Option<String>
) -> Result<Vec<String>, String> {
  let started = Instant::now();
  tracing::info!(
    request_id = ?request_id,
    "dictionary_languages command \
     invoked"
  );

  let result =
    (|| -> anyhow::Result<Vec<String>> {
      let (settings, conn, schema) =
        ensure_dictionary_ready()?;
      let Some(language_col) =
        schema.language_col.as_ref()
      else {
        let mut out = Vec::new();
        if let Some(default_language) =
          settings.default_language
        {
          out.push(default_language);
        }
        out.push("all".to_string());
        return Ok(out);
      };

      let sql = format!(
        "SELECT DISTINCT {lang} FROM \
         {table} WHERE {lang} IS NOT \
         NULL AND TRIM({lang}) <> '' \
         ORDER BY {lang} COLLATE \
         NOCASE LIMIT 1024",
        lang =
          quote_identifier(language_col),
        table =
          quote_identifier(&schema.table),
      );
      let mut stmt = conn.prepare(&sql)?;
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
          settings.default_language
      {
        values.push(default_language);
      }
      Ok(values)
    })();

  let elapsed_ms =
    started.elapsed().as_millis();
  if let Err(err) = result.as_ref() {
    tracing::error!(
      request_id = ?request_id,
      elapsed_ms,
      error = %err,
      "dictionary_languages command \
       failed"
    );
  } else {
    tracing::info!(
      request_id = ?request_id,
      elapsed_ms,
      count = result.as_ref().map_or(0, |langs| langs.len()),
      "dictionary_languages command \
       completed"
    );
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[tracing::instrument(skip(args), fields(request_id = ?request_id, query_len = args.query.len(), language = ?args.language))]
pub async fn dictionary_search(
  args: DictionarySearchArgs,
  request_id: Option<String>
) -> Result<DictionarySearchResult, String>
{
  let started = Instant::now();
  tracing::info!(
    request_id = ?request_id,
    query_len = args.query.len(),
    language = ?args.language,
    "dictionary_search command \
     invoked"
  );

  let result = (|| -> anyhow::Result<
    DictionarySearchResult,
  > {
    let (settings, conn, schema) =
      ensure_dictionary_ready()?;

    let query = args.query.trim();
    if query.is_empty() {
      return Ok(DictionarySearchResult {
        query: query.to_string(),
        language: normalized_language(
          args.language
        ),
        hits: vec![],
        total: 0,
        truncated: false,
        warnings: vec![
          "query is empty".to_string(),
        ]
      });
    }

    let requested_language =
      normalized_language(args.language)
        .or(
          settings.default_language
        );
    let limit = args
      .limit
      .unwrap_or(settings.max_results)
      .clamp(1, MAX_SEARCH_LIMIT);

    let table =
      quote_identifier(&schema.table);
    let word_col =
      quote_identifier(&schema.word_col);
    let id_col = schema.id_col.as_ref().map(
      |col| quote_identifier(col),
    );
    let language_col = schema
      .language_col
      .as_ref()
      .map(|col| quote_identifier(col));
    let pos_col = schema
      .part_of_speech_col
      .as_ref()
      .map(|col| quote_identifier(col));
    let pron_col = schema
      .pronunciation_col
      .as_ref()
      .map(|col| quote_identifier(col));
    let def_col = schema
      .definition_col
      .as_ref()
      .map(|col| quote_identifier(col));

    let select_id = id_col
      .as_deref()
      .unwrap_or("NULL");
    let select_lang = language_col
      .as_deref()
      .unwrap_or("NULL");
    let select_pos = pos_col
      .as_deref()
      .unwrap_or("NULL");
    let select_pron = pron_col
      .as_deref()
      .unwrap_or("NULL");
    let select_def = def_col
      .as_deref()
      .unwrap_or("NULL");

    let mut where_terms = vec![format!(
      "LOWER({word_col}) LIKE LOWER(?)"
    )];
    let mut params =
      vec![SqlValue::from(format!(
        "{query}%"
      ))];
    if let Some(language) =
      requested_language.as_ref()
      && let Some(lang_col) =
        language_col.as_ref()
    {
      where_terms.push(format!(
        "LOWER({lang_col}) = LOWER(?)"
      ));
      params
        .push(SqlValue::from(language.clone()));
    }

    let where_sql =
      where_terms.join(" AND ");
    let search_sql = format!(
      "SELECT {id}, {word}, {language}, \
       {pos}, {pron}, {summary} FROM \
       {table} WHERE {where_sql} ORDER \
       BY {word} COLLATE NOCASE LIMIT \
       ?",
      id = select_id,
      word = word_col,
      language = select_lang,
      pos = select_pos,
      pron = select_pron,
      summary = select_def
    );
    let mut search_params =
      params.clone();
    search_params
      .push(SqlValue::from(limit as i64));
    let mut stmt = conn.prepare(
      &search_sql
    )?;
    let hits = stmt
      .query_map(
        rusqlite::params_from_iter(
          search_params.iter()
        ),
        |row| {
          let id = row
            .get_ref(0)
            .ok()
            .and_then(value_ref_to_string)
            .and_then(|value| {
              value.parse::<i64>().ok()
            });
          let word = row
            .get_ref(1)
            .ok()
            .and_then(value_ref_to_string)
            .unwrap_or_default();
          let language = row
            .get_ref(2)
            .ok()
            .and_then(value_ref_to_string);
          let part_of_speech = row
            .get_ref(3)
            .ok()
            .and_then(value_ref_to_string);
          let pronunciation = row
            .get_ref(4)
            .ok()
            .and_then(value_ref_to_string);
          let summary = row
            .get_ref(5)
            .ok()
            .and_then(value_ref_to_string);
          Ok(DictionarySearchHit {
            id,
            word,
            language,
            part_of_speech,
            pronunciation,
            summary,
            source_table: schema
              .table
              .clone(),
            matched_by_prefix: true
          })
        },
      )?
      .collect::<Result<Vec<_>, _>>()?;

    let count_sql = format!(
      "SELECT COUNT(1) FROM {table} \
       WHERE {where_sql}"
    );
    let mut count_stmt = conn.prepare(
      &count_sql
    )?;
    let total: i64 = count_stmt.query_row(
      rusqlite::params_from_iter(
        params.iter()
      ),
      |row| row.get(0),
    )?;

    Ok(DictionarySearchResult {
      query: query.to_string(),
      language: requested_language,
      total: total.max(0) as u64,
      truncated: (hits.len() as u64)
        < total.max(0) as u64,
      hits,
      warnings: vec![]
    })
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
  } else if let Ok(payload) = result
    .as_ref()
  {
    tracing::info!(
      request_id = ?request_id,
      elapsed_ms,
      hits = payload.hits.len(),
      total = payload.total,
      "dictionary_search command \
       completed"
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

  let result =
    (|| -> anyhow::Result<
      Option<DictionaryEntry>,
    > {
      let (settings, conn, schema) =
        ensure_dictionary_ready()?;
      let requested_language =
        normalized_language(
          args.language
        )
        .or(settings.default_language);
      let requested_word =
        args.word.and_then(|word| {
          let trimmed = word.trim();
          if trimmed.is_empty() {
            None
          } else {
            Some(trimmed.to_string())
          }
        });

      if args.id.is_none()
        && requested_word.is_none()
      {
        anyhow::bail!(
          "dictionary_entry requires id \
           or word"
        );
      }

      let table =
        quote_identifier(&schema.table);
      let word_col =
        quote_identifier(&schema.word_col);
      let id_col = schema.id_col.as_ref().map(
        |col| quote_identifier(col),
      );
      let language_col = schema
        .language_col
        .as_ref()
        .map(|col| quote_identifier(col));
      let pos_col = schema
        .part_of_speech_col
        .as_ref()
        .map(|col| quote_identifier(col));
      let pron_col = schema
        .pronunciation_col
        .as_ref()
        .map(|col| quote_identifier(col));
      let def_col = schema
        .definition_col
        .as_ref()
        .map(|col| quote_identifier(col));
      let ety_col = schema
        .etymology_col
        .as_ref()
        .map(|col| quote_identifier(col));
      let ex_col = schema
        .example_col
        .as_ref()
        .map(|col| quote_identifier(col));
      let notes_col =
        schema.notes_col.as_ref().map(
          |col| quote_identifier(col),
        );

      let select_id = id_col
        .as_deref()
        .unwrap_or("NULL");
      let select_lang = language_col
        .as_deref()
        .unwrap_or("NULL");
      let select_pos = pos_col
        .as_deref()
        .unwrap_or("NULL");
      let select_pron = pron_col
        .as_deref()
        .unwrap_or("NULL");
      let select_def = def_col
        .as_deref()
        .unwrap_or("NULL");
      let select_ety = ety_col
        .as_deref()
        .unwrap_or("NULL");
      let select_ex = ex_col
        .as_deref()
        .unwrap_or("NULL");
      let select_notes = notes_col
        .as_deref()
        .unwrap_or("NULL");

      let mut where_terms = Vec::<
        String,
      >::new();
      let mut params = Vec::<SqlValue>::new();

      if let Some(id) = args.id {
        if let Some(id_col) = id_col.as_ref()
        {
          where_terms
            .push(format!("{id_col} = ?"));
          params.push(SqlValue::from(id));
        } else {
          tracing::warn!(
            "dictionary_entry called with \
             id, but discovered schema has \
             no id column"
          );
        }
      }

      if where_terms.is_empty()
        && let Some(word) =
          requested_word.as_ref()
      {
        where_terms.push(format!(
          "LOWER({word_col}) = LOWER(?)"
        ));
        params.push(SqlValue::from(
          word.clone()
        ));
      }

      if let Some(language) =
        requested_language.as_ref()
        && let Some(lang_col) =
          language_col.as_ref()
      {
        where_terms.push(format!(
          "LOWER({lang_col}) = LOWER(?)"
        ));
        params.push(SqlValue::from(
          language.clone()
        ));
      }

      if where_terms.is_empty() {
        anyhow::bail!(
          "dictionary_entry query criteria \
           could not be built from \
           available columns"
        );
      }

      let sql = format!(
        "SELECT {id}, {word}, {language}, \
         {pos}, {pron}, {defn}, \
         {etymology}, {example}, \
         {notes} FROM {table} WHERE \
         {where_sql} ORDER BY {word} \
         COLLATE NOCASE LIMIT 1",
        id = select_id,
        word = word_col,
        language = select_lang,
        pos = select_pos,
        pron = select_pron,
        defn = select_def,
        etymology = select_ety,
        example = select_ex,
        notes = select_notes,
        where_sql = where_terms.join(" \
                                     AND ")
      );

      let mut stmt = conn.prepare(&sql)?;
      let mut rows = stmt.query(
        rusqlite::params_from_iter(
          params.iter()
        ),
      )?;
      let Some(row) = rows.next()?
      else {
        return Ok(None);
      };

      let id = row
        .get_ref(0)
        .ok()
        .and_then(value_ref_to_string)
        .and_then(|value| {
          value.parse::<i64>().ok()
        });
      let word = row
        .get_ref(1)
        .ok()
        .and_then(value_ref_to_string)
        .unwrap_or_default();
      let language = row
        .get_ref(2)
        .ok()
        .and_then(value_ref_to_string);
      let part_of_speech = row
        .get_ref(3)
        .ok()
        .and_then(value_ref_to_string);
      let pronunciation = row
        .get_ref(4)
        .ok()
        .and_then(value_ref_to_string);
      let definitions = split_text_sections(
        row.get_ref(5)
          .ok()
          .and_then(value_ref_to_string),
      );
      let etymology = row
        .get_ref(6)
        .ok()
        .and_then(value_ref_to_string);
      let examples = split_text_sections(
        row.get_ref(7)
          .ok()
          .and_then(value_ref_to_string),
      );
      let notes = split_text_sections(
        row.get_ref(8)
          .ok()
          .and_then(value_ref_to_string),
      );

      Ok(Some(DictionaryEntry {
        id,
        word,
        language,
        part_of_speech,
        pronunciation,
        etymology,
        definitions,
        examples,
        notes,
        source_table: schema.table
      }))
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
      "dictionary_entry command \
       completed"
    );
  }

  result.map_err(err_to_string)
}
