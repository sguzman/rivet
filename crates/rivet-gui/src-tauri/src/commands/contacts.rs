use std::collections::{
  HashMap,
  HashSet
};
use std::path::{
  Path,
  PathBuf
};
use std::sync::{
  Mutex,
  OnceLock
};

use rivet_gui_shared::{
  ContactCreate,
  ContactDto,
  ContactFieldValue,
  ContactIdentityFingerprint,
  ContactIdArg,
  ContactImportBatch,
  ContactImportConflict,
  ContactsDedupeDecideArgs,
  ContactsDedupeDecideResult,
  ContactOpenActionArgs,
  ContactOpenActionResult,
  ContactPatch,
  ContactsDedupePreviewArgs,
  ContactsDedupePreviewResult,
  ContactsDeleteBulkArgs,
  ContactsImportCommitArgs,
  ContactsImportCommitResult,
  ContactsImportPreviewArgs,
  ContactsImportPreviewResult,
  ContactsListArgs,
  ContactsListResult,
  ContactsMergeArgs,
  ContactsMergeResult,
  ContactsMergeUndoArgs,
  ContactsMergeUndoResult,
  DedupDecision,
  MergeAudit,
  ContactUpdateArgs,
  ContactDedupeCandidateGroup,
};
use uuid::Uuid;

const CONTACTS_FILE: &str =
  "contacts.data";
const CONTACTS_DELETED_FILE: &str =
  "contacts_deleted.data";
const CONTACTS_IMPORT_BATCHES_FILE:
  &str = "contacts_import_batches.data";
const CONTACTS_MERGE_UNDO_FILE:
  &str = "contacts_merge_undo.data";
const CONTACTS_MERGE_AUDIT_FILE:
  &str = "contacts_merge_audit.data";
const CONTACTS_DEDUPE_DECISIONS_FILE:
  &str = "contacts_dedupe_decisions.data";
const CONTACTS_IMPORT_ERRORS_DIR:
  &str = "contacts_import_errors";

const CONTACTS_MAX_NAME_LEN: usize = 256;
const CONTACTS_MAX_NOTES_LEN: usize = 8_000;
const CONTACTS_MAX_ORG_LEN: usize = 256;
const CONTACTS_MAX_TITLE_LEN: usize = 256;
const CONTACTS_MAX_FIELD_KIND_LEN: usize = 64;
const CONTACTS_MAX_FIELD_VALUE_LEN: usize = 512;
const CONTACTS_MAX_MULTI_FIELDS: usize = 32;
const CONTACTS_MAX_ADDRESSES: usize = 8;
const CONTACTS_MAX_AVATAR_DATA_URL_LEN: usize =
  2_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContactsMergeUndoEntry {
  undo_id:         String,
  contacts_before: Vec<ContactDto>,
  created_at:      String,
}

fn contacts_lock() -> &'static Mutex<()> {
  static LOCK: OnceLock<Mutex<()>> =
    OnceLock::new();
  LOCK.get_or_init(|| Mutex::new(()))
}

#[derive(Debug, Default)]
struct ContactsIndexCache {
  path:         Option<PathBuf>,
  revision:     Option<u128>,
  contacts:     Vec<ContactDto>,
  search_blobs: Vec<String>,
  query_hits:
    HashMap<String, Vec<usize>>,
  fingerprints:
    Vec<ContactIdentityFingerprint>,
}

fn contacts_index_cache(
) -> &'static Mutex<ContactsIndexCache> {
  static CACHE: OnceLock<
    Mutex<ContactsIndexCache>,
  > = OnceLock::new();
  CACHE.get_or_init(|| {
    Mutex::new(
      ContactsIndexCache::default(),
    )
  })
}

fn file_revision(
  path: &Path
) -> Option<u128> {
  let metadata = std::fs::metadata(path).ok()?;
  let modified = metadata.modified().ok()?;
  let duration = modified
    .duration_since(
      std::time::UNIX_EPOCH
    )
    .ok()?;
  Some(duration.as_millis())
}

fn set_contacts_cache(
  contacts_path: &Path,
  contacts: &[ContactDto],
) -> anyhow::Result<()> {
  let mut cache = contacts_index_cache()
    .lock()
    .map_err(|_| {
      anyhow::anyhow!(
        "contacts cache lock poisoned"
      )
    })?;

  cache.path =
    Some(contacts_path.to_path_buf());
  cache.revision =
    file_revision(contacts_path);
  cache.contacts = contacts.to_vec();
  cache.search_blobs = contacts
    .iter()
    .map(build_contact_search_blob)
    .collect();
  cache.query_hits.clear();
  cache.fingerprints = contacts
    .iter()
    .map(build_contact_fingerprint)
    .collect();
  Ok(())
}

fn load_contacts_cached(
  contacts_path: &Path
) -> anyhow::Result<Vec<ContactDto>> {
  {
    let cache = contacts_index_cache()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts cache lock poisoned"
        )
      })?;
    let same_path = cache
      .path
      .as_deref()
      .is_some_and(|path| {
        path == contacts_path
      });
    if same_path
      && cache.revision
        == file_revision(contacts_path)
    {
      return Ok(cache.contacts.clone());
    }
  }

  let mut contacts =
    load_jsonl::<ContactDto>(
      contacts_path
    )?;
  for contact in &mut contacts {
    ensure_contact_defaults(contact);
  }
  sort_contacts(&mut contacts);
  set_contacts_cache(
    contacts_path,
    &contacts,
  )?;
  Ok(contacts)
}

fn resolve_contacts_data_dir() -> PathBuf {
  if let Ok(path) =
    std::env::var("RIVET_GUI_DATA")
  {
    return PathBuf::from(path);
  }

  if let Ok(cwd) = std::env::current_dir()
  {
    return cwd.join(".rivet_gui_data");
  }

  PathBuf::from(".rivet_gui_data")
}

fn ensure_contacts_store() -> anyhow::Result<
  (PathBuf, PathBuf, PathBuf, PathBuf)
> {
  let dir = resolve_contacts_data_dir();
  std::fs::create_dir_all(&dir)
    .with_context(|| {
      format!(
        "failed to create contacts data \
         dir {}",
        dir.display()
      )
    })?;

  let contacts = dir.join(CONTACTS_FILE);
  let deleted =
    dir.join(CONTACTS_DELETED_FILE);
  let batches =
    dir.join(CONTACTS_IMPORT_BATCHES_FILE);
  let merge_undo =
    dir.join(CONTACTS_MERGE_UNDO_FILE);
  let merge_audit =
    dir.join(CONTACTS_MERGE_AUDIT_FILE);
  let dedupe_decisions =
    dir.join(
      CONTACTS_DEDUPE_DECISIONS_FILE,
    );
  let import_errors_dir =
    dir.join(CONTACTS_IMPORT_ERRORS_DIR);

  for path in [
    &contacts,
    &deleted,
    &batches,
    &merge_undo,
    &merge_audit,
    &dedupe_decisions,
  ] {
    if !path.exists() {
      std::fs::write(path, "")
        .with_context(|| {
          format!(
            "failed to initialize {}",
            path.display()
          )
        })?;
    }
  }
  std::fs::create_dir_all(
    &import_errors_dir,
  )
  .with_context(|| {
    format!(
      "failed to create import error \
       directory {}",
      import_errors_dir.display()
    )
  })?;

  Ok((
    contacts, deleted, batches,
    merge_undo,
  ))
}

fn contacts_merge_audit_path(
  contacts_path: &Path
) -> anyhow::Result<PathBuf> {
  let Some(parent) = contacts_path.parent()
  else {
    anyhow::bail!(
      "failed to resolve contacts data \
       directory"
    );
  };
  Ok(parent.join(CONTACTS_MERGE_AUDIT_FILE))
}

fn contacts_dedupe_decisions_path(
  contacts_path: &Path
) -> anyhow::Result<PathBuf> {
  let Some(parent) = contacts_path.parent()
  else {
    anyhow::bail!(
      "failed to resolve contacts data \
       directory"
    );
  };
  Ok(parent.join(
    CONTACTS_DEDUPE_DECISIONS_FILE
  ))
}

fn load_jsonl<T>(
  path: &Path
) -> anyhow::Result<Vec<T>>
where
  T: for<'de> Deserialize<'de>,
{
  let raw =
    std::fs::read_to_string(path)
      .with_context(|| {
        format!(
          "failed to read {}",
          path.display()
        )
      })?;

  let mut items = Vec::<T>::new();
  for (index, line) in
    raw.lines().enumerate()
  {
    let token = line.trim();
    if token.is_empty() {
      continue;
    }
    let item = serde_json::from_str::<T>(
      token,
    )
    .with_context(|| {
      format!(
        "failed parsing {} line {}",
        path.display(),
        index + 1
      )
    })?;
    items.push(item);
  }
  Ok(items)
}

fn save_jsonl<T>(
  path: &Path,
  items: &[T],
) -> anyhow::Result<()>
where
  T: Serialize,
{
  let mut out = String::new();
  for item in items {
    out.push_str(
      &serde_json::to_string(item)
        .map_err(anyhow::Error::new)?,
    );
    out.push('\n');
  }

  std::fs::write(path, out)
    .with_context(|| {
      format!(
        "failed writing {}",
        path.display()
      )
    })
}

fn append_jsonl<T>(
  path: &Path,
  item: &T,
) -> anyhow::Result<()>
where
  T: Serialize,
{
  use std::io::Write;

  let mut file = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)
    .with_context(|| {
      format!(
        "failed opening {}",
        path.display()
      )
    })?;
  writeln!(
    file,
    "{}",
    serde_json::to_string(item)
      .map_err(anyhow::Error::new)?
  )
  .map_err(anyhow::Error::new)
  .with_context(|| {
    format!(
      "failed appending {}",
      path.display()
    )
  })
}

fn now_iso() -> String {
  Utc::now().to_rfc3339()
}

fn fold_diacritic(
  ch: char
) -> &'static str {
  match ch {
    | 'à'
    | 'á'
    | 'â'
    | 'ã'
    | 'ä'
    | 'å'
    | 'ā'
    | 'ă'
    | 'ą' => "a",
    | 'ç'
    | 'ć'
    | 'č'
    | 'ĉ'
    | 'ċ' => "c",
    | 'ď'
    | 'đ' => "d",
    | 'è'
    | 'é'
    | 'ê'
    | 'ë'
    | 'ē'
    | 'ĕ'
    | 'ė'
    | 'ę'
    | 'ě' => "e",
    | 'ƒ' => "f",
    | 'ĝ'
    | 'ğ'
    | 'ġ'
    | 'ģ' => "g",
    | 'ĥ'
    | 'ħ' => "h",
    | 'ì'
    | 'í'
    | 'î'
    | 'ï'
    | 'ĩ'
    | 'ī'
    | 'ĭ'
    | 'į'
    | 'ı' => "i",
    | 'ĵ' => "j",
    | 'ķ' => "k",
    | 'ĺ'
    | 'ļ'
    | 'ľ'
    | 'ŀ'
    | 'ł' => "l",
    | 'ñ'
    | 'ń'
    | 'ņ'
    | 'ň' => "n",
    | 'ò'
    | 'ó'
    | 'ô'
    | 'õ'
    | 'ö'
    | 'ø'
    | 'ō'
    | 'ŏ'
    | 'ő' => "o",
    | 'ŕ'
    | 'ŗ'
    | 'ř' => "r",
    | 'ś'
    | 'ŝ'
    | 'ş'
    | 'š' => "s",
    | 'ß' => "ss",
    | 'ť'
    | 'ţ'
    | 'ŧ' => "t",
    | 'ù'
    | 'ú'
    | 'û'
    | 'ü'
    | 'ũ'
    | 'ū'
    | 'ŭ'
    | 'ů'
    | 'ű'
    | 'ų' => "u",
    | 'ŵ' => "w",
    | 'ý'
    | 'ÿ'
    | 'ŷ' => "y",
    | 'ź'
    | 'ż'
    | 'ž' => "z",
    | 'æ' => "ae",
    | 'œ' => "oe",
    | _ => "",
  }
}

fn normalize_text(
  value: &str
) -> String {
  let mut normalized =
    String::with_capacity(
      value.len(),
    );
  for ch in value.chars() {
    for lower in ch.to_lowercase() {
      let folded =
        fold_diacritic(lower);
      if !folded.is_empty() {
        normalized
          .push_str(folded);
      } else {
        normalized.push(lower);
      }
    }
  }

  normalized
    .trim()
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn normalize_phone(
  value: &str
) -> String {
  let mut out = String::new();
  for ch in value.chars() {
    if ch.is_ascii_digit() {
      out.push(ch);
    } else if ch == '+'
      && out.is_empty()
    {
      out.push(ch);
    }
  }
  out
}

fn normalize_email(
  value: &str
) -> String {
  normalize_text(value)
}

fn sort_contacts(
  contacts: &mut [ContactDto]
) {
  contacts.sort_by(|a, b| {
    normalize_text(&a.display_name)
      .cmp(&normalize_text(
        &b.display_name,
      ))
      .then_with(|| {
        b.updated_at.cmp(
          &a.updated_at,
        )
      })
  });
}

fn build_contact_search_blob(
  contact: &ContactDto
) -> String {
  let mut haystacks = vec![
    normalize_text(&contact.display_name),
    normalize_text(
      contact
        .given_name
        .as_deref()
        .unwrap_or_default(),
    ),
    normalize_text(
      contact
        .family_name
        .as_deref()
        .unwrap_or_default(),
    ),
    normalize_text(
      contact
        .nickname
        .as_deref()
        .unwrap_or_default(),
    ),
    normalize_text(
      contact
        .notes
        .as_deref()
        .unwrap_or_default(),
    ),
    normalize_text(
      contact
        .organization
        .as_deref()
        .unwrap_or_default(),
    ),
  ];
  for field in &contact.emails {
    haystacks.push(normalize_email(
      &field.value,
    ));
  }
  for field in &contact.phones {
    haystacks.push(normalize_phone(
      &field.value,
    ));
  }
  haystacks.join(" ")
}

fn build_contact_fingerprint(
  contact: &ContactDto
) -> ContactIdentityFingerprint {
  ContactIdentityFingerprint {
    contact_id: contact.id,
    name_key: contact_name_key(contact),
    email_hashes: contact
      .emails
      .iter()
      .map(|item| {
        normalize_email(&item.value)
      })
      .filter(|item| {
        !item.is_empty()
      })
      .collect(),
    phone_hashes: contact
      .phones
      .iter()
      .map(|item| {
        normalize_phone(&item.value)
      })
      .filter(|item| {
        !item.is_empty()
      })
      .collect(),
  }
}

fn contact_name_key(
  contact: &ContactDto
) -> String {
  let mut parts = Vec::<String>::new();
  if let Some(given) =
    contact.given_name.as_deref()
  {
    if !given.trim().is_empty() {
      parts.push(
        normalize_text(given)
      );
    }
  }
  if let Some(family) =
    contact.family_name.as_deref()
  {
    if !family.trim().is_empty() {
      parts.push(
        normalize_text(family)
      );
    }
  }

  if parts.is_empty() {
    return normalize_text(
      &contact.display_name,
    );
  }

  parts.join(" ")
}

fn contact_domains(
  contact: &ContactDto
) -> HashSet<String> {
  let mut out = HashSet::<String>::new();
  for email in &contact.emails {
    let token =
      normalize_email(&email.value);
    if let Some((_, domain)) =
      token.split_once('@')
    {
      if !domain.trim().is_empty() {
        out.insert(
          domain.to_string(),
        );
      }
    }
  }
  out
}

fn levenshtein_distance(
  left: &str,
  right: &str,
) -> usize {
  if left == right {
    return 0;
  }
  if left.is_empty() {
    return right.chars().count();
  }
  if right.is_empty() {
    return left.chars().count();
  }

  let right_chars =
    right.chars().collect::<Vec<_>>();
  let mut previous =
    (0..=right_chars.len())
      .collect::<Vec<_>>();
  let mut current =
    vec![0_usize; right_chars.len() + 1];

  for (left_index, left_char) in
    left.chars().enumerate()
  {
    current[0] = left_index + 1;
    for (
      right_index,
      right_char,
    ) in right_chars
      .iter()
      .enumerate()
    {
      let substitution_cost =
        if left_char == *right_char {
          0
        } else {
          1
        };
      let deletion =
        previous[right_index + 1] + 1;
      let insertion =
        current[right_index] + 1;
      let substitution =
        previous[right_index]
          + substitution_cost;
      current[right_index + 1] =
        deletion
          .min(insertion)
          .min(substitution);
    }
    previous.copy_from_slice(
      &current,
    );
  }

  previous[right_chars.len()]
}

fn normalized_name_similarity(
  left: &str,
  right: &str,
) -> f32 {
  if left.trim().is_empty()
    || right.trim().is_empty()
  {
    return 0.0;
  }

  let distance =
    levenshtein_distance(left, right)
      as f32;
  let max_len = left
    .chars()
    .count()
    .max(right.chars().count())
    as f32;
  if max_len <= 0.0 {
    return 1.0;
  }

  (1.0 - (distance / max_len))
    .clamp(0.0, 1.0)
}

fn ensure_contact_defaults(
  contact: &mut ContactDto
) {
  for group in [
    &mut contact.phones,
    &mut contact.emails,
    &mut contact.websites,
  ] {
    group.retain(|field| {
      !field.value.trim().is_empty()
    });
    if group.is_empty() {
      continue;
    }
    let mut first_primary = None;
    for (index, field) in
      group.iter().enumerate()
    {
      if field.is_primary {
        first_primary = Some(index);
        break;
      }
    }
    let primary_index =
      first_primary.unwrap_or(0);
    for (index, field) in
      group.iter_mut().enumerate()
    {
      field.is_primary =
        index == primary_index;
    }
  }

  if contact
    .source_id
    .trim()
    .is_empty()
  {
    contact.source_id =
      "local".to_string();
  }
  if contact
    .source_kind
    .trim()
    .is_empty()
  {
    contact.source_kind =
      "local".to_string();
  }
  if contact
    .import_batch_id
    .as_deref()
    .is_some_and(|value| {
      value.trim().is_empty()
    })
  {
    contact.import_batch_id = None;
  }
  if contact
    .source_file_name
    .as_deref()
    .is_some_and(|value| {
      value.trim().is_empty()
    })
  {
    contact.source_file_name = None;
  }

  if contact
    .display_name
    .trim()
    .is_empty()
  {
    let full_name = [
      contact
        .given_name
        .as_deref()
        .unwrap_or_default()
        .trim(),
      contact
        .family_name
        .as_deref()
        .unwrap_or_default()
        .trim(),
    ]
    .iter()
    .filter(|part| {
      !part.is_empty()
    })
    .copied()
    .collect::<Vec<_>>()
    .join(" ");

    if !full_name.is_empty() {
      contact.display_name =
        full_name;
    } else if let Some(email) =
      contact.emails.first()
    {
      contact.display_name =
        email.value.clone();
    } else if let Some(phone) =
      contact.phones.first()
    {
      contact.display_name =
        phone.value.clone();
    }
  }
}

fn validate_contact(
  contact: &ContactDto
) -> anyhow::Result<()> {
  let has_name = !contact
    .display_name
    .trim()
    .is_empty();
  let has_email = contact
    .emails
    .iter()
    .any(|value| {
      !value.value.trim().is_empty()
    });
  let has_phone = contact
    .phones
    .iter()
    .any(|value| {
      !value.value.trim().is_empty()
    });

  if !(has_name || has_email || has_phone) {
    anyhow::bail!(
      "contact requires at least a \
       name, email, or phone"
    );
  }

  if contact.display_name.len()
    > CONTACTS_MAX_NAME_LEN
  {
    anyhow::bail!(
      "display_name exceeds {} \
       characters",
      CONTACTS_MAX_NAME_LEN
    );
  }

  if contact
    .notes
    .as_deref()
    .is_some_and(|value| {
      value.len()
        > CONTACTS_MAX_NOTES_LEN
    })
  {
    anyhow::bail!(
      "notes exceed {} characters",
      CONTACTS_MAX_NOTES_LEN
    );
  }
  if contact
    .organization
    .as_deref()
    .is_some_and(|value| {
      value.len()
        > CONTACTS_MAX_ORG_LEN
    })
  {
    anyhow::bail!(
      "organization exceeds {} \
       characters",
      CONTACTS_MAX_ORG_LEN
    );
  }
  if contact
    .title
    .as_deref()
    .is_some_and(|value| {
      value.len()
        > CONTACTS_MAX_TITLE_LEN
    })
  {
    anyhow::bail!(
      "title exceeds {} characters",
      CONTACTS_MAX_TITLE_LEN
    );
  }
  if contact
    .avatar_data_url
    .as_deref()
    .is_some_and(|value| {
      value.len()
        > CONTACTS_MAX_AVATAR_DATA_URL_LEN
    })
  {
    anyhow::bail!(
      "avatar data exceeds {} \
       characters",
      CONTACTS_MAX_AVATAR_DATA_URL_LEN
    );
  }

  for (label, fields) in [
    ("phones", &contact.phones),
    ("emails", &contact.emails),
    ("websites", &contact.websites),
  ] {
    if fields.len()
      > CONTACTS_MAX_MULTI_FIELDS
    {
      anyhow::bail!(
        "{label} exceeds maximum \
         count {}",
        CONTACTS_MAX_MULTI_FIELDS
      );
    }
    for field in fields {
      if field.kind.len()
        > CONTACTS_MAX_FIELD_KIND_LEN
      {
        anyhow::bail!(
          "{label}.kind exceeds {} \
           characters",
          CONTACTS_MAX_FIELD_KIND_LEN
        );
      }
      if field.value.len()
        > CONTACTS_MAX_FIELD_VALUE_LEN
      {
        anyhow::bail!(
          "{label}.value exceeds {} \
           characters",
          CONTACTS_MAX_FIELD_VALUE_LEN
        );
      }
    }
  }

  if contact.addresses.len()
    > CONTACTS_MAX_ADDRESSES
  {
    anyhow::bail!(
      "addresses exceed maximum \
       count {}",
      CONTACTS_MAX_ADDRESSES
    );
  }
  for address in &contact.addresses {
    for (label, value) in [
      ("kind", &address.kind),
      ("street", &address.street),
      ("city", &address.city),
      ("region", &address.region),
      (
        "postal_code",
        &address.postal_code,
      ),
      ("country", &address.country),
    ] {
      if value.len()
        > CONTACTS_MAX_FIELD_VALUE_LEN
      {
        anyhow::bail!(
          "addresses.{label} exceeds \
           {} characters",
          CONTACTS_MAX_FIELD_VALUE_LEN
        );
      }
    }
  }

  for email in &contact.emails {
    let token = email.value.trim();
    if token.is_empty() {
      continue;
    }
    if !token.contains('@')
      || token.starts_with('@')
      || token.ends_with('@')
    {
      anyhow::bail!(
        "invalid email address: \
         {token}"
      );
    }
  }

  Ok(())
}

fn from_create_payload(
  create: ContactCreate,
  existing_id: Option<Uuid>,
  existing_created_at: Option<String>,
) -> ContactDto {
  ContactDto {
    id: existing_id
      .unwrap_or_else(Uuid::new_v4),
    display_name: create
      .display_name
      .unwrap_or_default(),
    avatar_data_url: create
      .avatar_data_url,
    import_batch_id: create
      .import_batch_id,
    source_file_name: create
      .source_file_name,
    given_name: create.given_name,
    family_name: create.family_name,
    nickname: create.nickname,
    notes: create.notes,
    phones: create.phones,
    emails: create.emails,
    websites: create.websites,
    birthday: create.birthday,
    organization: create.organization,
    title: create.title,
    addresses: create.addresses,
    source_id: create
      .source_id
      .unwrap_or_else(|| {
        "local".to_string()
      }),
    source_kind: create
      .source_kind
      .unwrap_or_else(|| {
        "local".to_string()
      }),
    remote_id: create.remote_id,
    link_group_id: create
      .link_group_id,
    created_at: existing_created_at
      .unwrap_or_else(now_iso),
    updated_at: now_iso(),
  }
}

fn apply_contact_patch(
  contact: &mut ContactDto,
  patch: ContactPatch,
) {
  if let Some(display_name) =
    patch.display_name
  {
    contact.display_name =
      display_name
        .unwrap_or_default();
  }
  if let Some(avatar_data_url) =
    patch.avatar_data_url
  {
    contact.avatar_data_url =
      avatar_data_url;
  }
  if let Some(import_batch_id) =
    patch.import_batch_id
  {
    contact.import_batch_id =
      import_batch_id;
  }
  if let Some(source_file_name) =
    patch.source_file_name
  {
    contact.source_file_name =
      source_file_name;
  }
  if let Some(given_name) =
    patch.given_name
  {
    contact.given_name =
      given_name;
  }
  if let Some(family_name) =
    patch.family_name
  {
    contact.family_name =
      family_name;
  }
  if let Some(nickname) =
    patch.nickname
  {
    contact.nickname = nickname;
  }
  if let Some(notes) = patch.notes {
    contact.notes = notes;
  }
  if let Some(phones) = patch.phones {
    contact.phones = phones;
  }
  if let Some(emails) = patch.emails {
    contact.emails = emails;
  }
  if let Some(websites) =
    patch.websites
  {
    contact.websites = websites;
  }
  if let Some(birthday) =
    patch.birthday
  {
    contact.birthday = birthday;
  }
  if let Some(organization) =
    patch.organization
  {
    contact.organization =
      organization;
  }
  if let Some(title) = patch.title {
    contact.title = title;
  }
  if let Some(addresses) =
    patch.addresses
  {
    contact.addresses = addresses;
  }
  if let Some(source_id) =
    patch.source_id
  {
    contact.source_id =
      source_id
        .unwrap_or_default();
  }
  if let Some(source_kind) =
    patch.source_kind
  {
    contact.source_kind =
      source_kind
        .unwrap_or_default();
  }
  if let Some(remote_id) =
    patch.remote_id
  {
    contact.remote_id = remote_id;
  }
  if let Some(link_group_id) =
    patch.link_group_id
  {
    contact.link_group_id =
      link_group_id;
  }

  contact.updated_at = now_iso();
}

fn contact_matches_query(
  contact: &ContactDto,
  query: &str,
) -> bool {
  if query.trim().is_empty() {
    return true;
  }
  let q = normalize_text(query);
  build_contact_search_blob(contact)
    .contains(&q)
}

fn parse_updated_after(
  updated_after: Option<&str>
) -> anyhow::Result<
  Option<DateTime<Utc>>,
> {
  let Some(token) = updated_after else {
    return Ok(None);
  };

  let parsed = DateTime::parse_from_rfc3339(
    token,
  )
  .map_err(anyhow::Error::new)
  .with_context(|| {
    format!(
      "invalid updated_after value: \
       {token}"
    )
  })?
  .with_timezone(&Utc);
  Ok(Some(parsed))
}

fn contacts_list_internal(
  args: ContactsListArgs,
) -> anyhow::Result<
  ContactsListResult,
> {
  let _guard = contacts_lock()
    .lock()
    .map_err(|_| {
      anyhow::anyhow!(
        "contacts store lock poisoned"
      )
    })?;
  let (
    contacts_path,
    _deleted_path,
    _batch_path,
    _undo_path,
  ) = ensure_contacts_store()?;
  let _ = load_contacts_cached(
    &contacts_path
  )?;

  let updated_after =
    parse_updated_after(
      args.updated_after.as_deref(),
    )?;

  let query_key = args
    .query
    .as_deref()
    .map(normalize_text)
    .filter(|token| {
      !token.is_empty()
    });

  let base_contacts = {
    let mut cache =
      contacts_index_cache()
        .lock()
        .map_err(|_| {
          anyhow::anyhow!(
            "contacts cache lock poisoned"
          )
        })?;
    let indices =
      if let Some(query) =
        query_key.as_ref()
      {
        if let Some(cached) = cache
          .query_hits
          .get(query)
        {
          cached.clone()
        } else {
          let built = cache
            .search_blobs
            .iter()
            .enumerate()
            .filter_map(
              |(index, blob)| {
                if blob.contains(query) {
                  Some(index)
                } else {
                  None
                }
              },
            )
            .collect::<Vec<_>>();
          cache.query_hits.insert(
            query.clone(),
            built.clone(),
          );
          built
        }
      } else {
        (0..cache.contacts.len())
          .collect::<Vec<_>>()
      };

    indices
      .into_iter()
      .filter_map(|index| {
        cache.contacts.get(index).cloned()
      })
      .collect::<Vec<_>>()
  };

  let mut contacts = base_contacts;

  contacts.retain(|contact| {
    if let Some(source) =
      args.source.as_deref()
      && contact.source_kind
        != source
      && contact.source_id != source
    {
      return false;
    }

    if let Some(after) =
      updated_after.as_ref()
    {
      if let Ok(updated) =
        DateTime::parse_from_rfc3339(
          &contact.updated_at,
        )
      {
        if updated.with_timezone(&Utc)
          < *after
        {
          return false;
        }
      }
    }

    true
  });

  let total = contacts.len();
  let limit = args
    .limit
    .unwrap_or(200)
    .clamp(1, 200);
  let offset = args
    .cursor
    .as_deref()
    .and_then(|token| {
      token.parse::<usize>().ok()
    })
    .unwrap_or(0)
    .min(total);

  let page = contacts
    .into_iter()
    .skip(offset)
    .take(limit)
    .collect::<Vec<_>>();
  let consumed = offset + page.len();
  let next_cursor =
    if consumed < total {
      Some(consumed.to_string())
    } else {
      None
    };

  Ok(ContactsListResult {
    contacts: page,
    next_cursor,
    total,
  })
}

fn score_pair(
  left: &ContactDto,
  right: &ContactDto,
) -> Option<(u32, String)> {
  let left_emails = left
    .emails
    .iter()
    .map(|item| {
      normalize_email(&item.value)
    })
    .filter(|item| {
      !item.is_empty()
    })
    .collect::<BTreeSet<_>>();
  let right_emails = right
    .emails
    .iter()
    .map(|item| {
      normalize_email(&item.value)
    })
    .filter(|item| {
      !item.is_empty()
    })
    .collect::<BTreeSet<_>>();

  if left_emails
    .intersection(&right_emails)
    .next()
    .is_some()
  {
    return Some((
      100,
      "exact email match"
        .to_string(),
    ));
  }

  let left_phones = left
    .phones
    .iter()
    .map(|item| {
      normalize_phone(&item.value)
    })
    .filter(|item| {
      !item.is_empty()
    })
    .collect::<BTreeSet<_>>();
  let right_phones = right
    .phones
    .iter()
    .map(|item| {
      normalize_phone(&item.value)
    })
    .filter(|item| {
      !item.is_empty()
    })
    .collect::<BTreeSet<_>>();

  if left_phones
    .intersection(&right_phones)
    .next()
    .is_some()
  {
    return Some((
      100,
      "exact phone match"
        .to_string(),
    ));
  }

  let left_name = contact_name_key(left);
  let right_name =
    contact_name_key(right);
  if !left_name.is_empty()
    && left_name == right_name
  {
    let left_org = normalize_text(
      left
        .organization
        .as_deref()
        .unwrap_or_default(),
    );
    let right_org = normalize_text(
      right
        .organization
        .as_deref()
        .unwrap_or_default(),
    );

    if !left_org.is_empty()
      && left_org == right_org
    {
      return Some((
        70,
        "same full name + org"
          .to_string(),
      ));
    }

    let left_domains =
      contact_domains(left);
    let right_domains =
      contact_domains(right);
    if left_domains
      .intersection(&right_domains)
      .next()
      .is_some()
    {
      return Some((
        60,
        "same full name + email \
         domain"
          .to_string(),
      ));
    }

    return Some((
      30,
      "fuzzy name similarity"
        .to_string(),
    ));
  }

  if !left_name.is_empty()
    && !right_name.is_empty()
  {
    let similarity =
      normalized_name_similarity(
        &left_name,
        &right_name,
      );
    if similarity >= 0.80 {
      let left_org = normalize_text(
        left
          .organization
          .as_deref()
          .unwrap_or_default(),
      );
      let right_org = normalize_text(
        right
          .organization
          .as_deref()
          .unwrap_or_default(),
      );

      if !left_org.is_empty()
        && left_org == right_org
      {
        return Some((
          65,
          "fuzzy name + org"
            .to_string(),
        ));
      }

      let left_domains =
        contact_domains(left);
      let right_domains =
        contact_domains(right);
      if left_domains
        .intersection(&right_domains)
        .next()
        .is_some()
      {
        return Some((
          62,
          "fuzzy name + email \
           domain"
            .to_string(),
        ));
      }

      return Some((
        30,
        "fuzzy name similarity"
          .to_string(),
      ));
    }
  }

  None
}

#[derive(Debug)]
struct Dsu {
  parent: Vec<usize>,
  rank:   Vec<u8>,
}

impl Dsu {
  fn new(size: usize) -> Self {
    Self {
      parent: (0..size).collect(),
      rank:   vec![0; size],
    }
  }

  fn find(
    &mut self,
    index: usize,
  ) -> usize {
    if self.parent[index] != index {
      let parent =
        self.parent[index];
      self.parent[index] =
        self.find(parent);
    }
    self.parent[index]
  }

  fn union(
    &mut self,
    left: usize,
    right: usize,
  ) {
    let mut a = self.find(left);
    let mut b = self.find(right);
    if a == b {
      return;
    }

    if self.rank[a] < self.rank[b] {
      std::mem::swap(
        &mut a,
        &mut b,
      );
    }

    self.parent[b] = a;
    if self.rank[a] == self.rank[b] {
      self.rank[a] += 1;
    }
  }
}

fn load_dedupe_decisions_map(
  path: &Path
) -> anyhow::Result<
  HashMap<String, DedupDecision>,
> {
  let decisions =
    load_jsonl::<DedupDecision>(path)?;
  let mut by_group =
    HashMap::<String, DedupDecision>::new();
  for decision in decisions {
    by_group.insert(
      decision.candidate_group_id.clone(),
      decision,
    );
  }
  Ok(by_group)
}

fn dedupe_groups(
  contacts: &[ContactDto],
  query: Option<&str>,
  decisions: &HashMap<String, DedupDecision>,
) -> Vec<ContactDedupeCandidateGroup> {
  let items = contacts
    .iter()
    .filter(|contact| {
      if let Some(query) = query {
        return contact_matches_query(
          contact, query,
        );
      }
      true
    })
    .cloned()
    .collect::<Vec<_>>();

  if items.len() < 2 {
    return Vec::new();
  }

  let mut dsu = Dsu::new(items.len());
  let mut pair_scores = HashMap::<
    (usize, usize),
    (u32, String),
  >::new();

  for left in 0..items.len() {
    for right in
      (left + 1)..items.len()
    {
      if let Some((score, reason)) =
        score_pair(
          &items[left],
          &items[right],
        )
      {
        if score >= 60 {
          dsu.union(left, right);
          pair_scores
            .insert((left, right), (
              score,
              reason,
            ));
        }
      }
    }
  }

  let mut grouped =
    BTreeMap::<usize, Vec<usize>>::new();
  for index in 0..items.len() {
    let root = dsu.find(index);
    grouped
      .entry(root)
      .or_default()
      .push(index);
  }

  let mut out = Vec::<
    ContactDedupeCandidateGroup,
  >::new();

  for (_root, members) in grouped {
    if members.len() < 2 {
      continue;
    }

    let mut best = (0_u32, String::new());
    for left_pos in 0..members.len() {
      for right_pos in
        (left_pos + 1)..members.len()
      {
        let left =
          members[left_pos];
        let right =
          members[right_pos];
        let key = if left < right {
          (left, right)
        } else {
          (right, left)
        };
        if let Some(pair) =
          pair_scores.get(&key)
          && pair.0 > best.0
        {
          best = pair.clone();
        }
      }
    }

    let mut group_ids = members
      .iter()
      .map(|index| {
        items[*index].id.to_string()
      })
      .collect::<Vec<_>>();
    group_ids.sort();
    let group_id = format!(
      "group:{}",
      group_ids.join(",")
    );
    if let Some(decision) =
      decisions.get(&group_id)
    {
      let normalized_decision = decision
        .decision
        .trim()
        .to_ascii_lowercase();
      if normalized_decision
        == "ignored"
        || normalized_decision
          == "separate"
        || normalized_decision
          == "merged"
      {
        continue;
      }
    }

    out.push(
      ContactDedupeCandidateGroup {
        group_id,
        reason: if best.1.is_empty() {
          "possible duplicate"
            .to_string()
        } else {
          best.1
        },
        score: best.0,
        contacts: members
          .into_iter()
          .map(|index| {
            items[index].clone()
          })
          .collect(),
      },
    );
  }

  out.sort_by(|a, b| {
    b.score
      .cmp(&a.score)
      .then_with(|| {
        b.contacts
          .len()
          .cmp(&a.contacts.len())
      })
  });

  out
}

fn merge_field_values(
  target: &mut Vec<ContactFieldValue>,
  source: &[ContactFieldValue],
) {
  let mut has_primary =
    target.iter().any(|item| {
      item.is_primary
    });
  let mut seen = target
    .iter()
    .map(|item| {
      normalize_text(
        &format!(
          "{}:{}",
          item.kind,
          item.value
        ),
      )
    })
    .collect::<HashSet<_>>();

  for field in source {
    if field.value.trim().is_empty() {
      continue;
    }
    let key = normalize_text(
      &format!(
        "{}:{}",
        field.kind,
        field.value
      ),
    );
    if seen.insert(key) {
      let mut next = field.clone();
      if next.is_primary
        && !has_primary
      {
        for existing in
          target.iter_mut()
        {
          existing.is_primary =
            false;
        }
        has_primary = true;
      } else if has_primary {
        next.is_primary = false;
      }
      target.push(next);
    }
  }

  if !target.iter().any(|item| {
    item.is_primary
  }) && !target.is_empty()
  {
    target[0].is_primary = true;
  }
}

fn merge_optional_text(
  target: &mut Option<String>,
  source: &Option<String>,
) {
  let missing = target
    .as_deref()
    .is_none_or(|token| {
      token.trim().is_empty()
    });
  if missing
    && let Some(value) = source
    && !value.trim().is_empty()
  {
    *target = Some(value.clone());
  }
}

fn merge_contact_records(
  target: &mut ContactDto,
  source: &ContactDto,
) {
  if target.display_name.trim().is_empty()
    && !source.display_name.trim().is_empty()
  {
    target.display_name =
      source.display_name.clone();
  }
  merge_optional_text(
    &mut target.given_name,
    &source.given_name,
  );
  merge_optional_text(
    &mut target.family_name,
    &source.family_name,
  );
  merge_optional_text(
    &mut target.nickname,
    &source.nickname,
  );
  merge_optional_text(
    &mut target.notes,
    &source.notes,
  );
  merge_optional_text(
    &mut target.avatar_data_url,
    &source.avatar_data_url,
  );
  merge_optional_text(
    &mut target.import_batch_id,
    &source.import_batch_id,
  );
  merge_optional_text(
    &mut target.source_file_name,
    &source.source_file_name,
  );
  merge_optional_text(
    &mut target.birthday,
    &source.birthday,
  );
  merge_optional_text(
    &mut target.organization,
    &source.organization,
  );
  merge_optional_text(
    &mut target.title,
    &source.title,
  );

  merge_field_values(
    &mut target.phones,
    &source.phones,
  );
  merge_field_values(
    &mut target.emails,
    &source.emails,
  );
  merge_field_values(
    &mut target.websites,
    &source.websites,
  );

  if target.addresses.is_empty() {
    target.addresses =
      source.addresses.clone();
  }

  if target
    .remote_id
    .as_deref()
    .is_none_or(|value| {
      value.trim().is_empty()
    })
  {
    target.remote_id =
      source.remote_id.clone();
  }

  if target
    .link_group_id
    .as_deref()
    .is_none_or(|value| {
      value.trim().is_empty()
    })
  {
    target.link_group_id =
      source.link_group_id.clone();
  }

  target.updated_at = now_iso();
}

fn normalize_vcard_kind(
  token: &str,
  fallback: &str,
) -> String {
  let lowered = token
    .trim()
    .trim_matches('"')
    .to_ascii_lowercase();
  if lowered.is_empty() {
    return fallback.to_string();
  }
  let lowered = lowered
    .trim_start_matches("x-")
    .trim_start_matches("ablabel=");
  if lowered.contains("home") {
    return "home".to_string();
  }
  if lowered.contains("work") {
    return "work".to_string();
  }
  if lowered.contains("cell")
    || lowered.contains("mobile")
    || lowered.contains("iphone")
  {
    return "mobile".to_string();
  }
  if lowered.contains("main")
    || lowered.contains("pref")
    || lowered.contains("primary")
    || lowered.contains("internet")
  {
    return fallback.to_string();
  }
  if lowered.contains("other") {
    return "other".to_string();
  }
  lowered.to_string()
}

fn parse_vcard_primary(
  header: &str
) -> bool {
  let lowered =
    header.to_ascii_lowercase();
  lowered.contains("pref")
    || lowered.contains("primary")
}

fn parse_vcard_type(
  header: &str,
  fallback: &str,
) -> String {
  let mut candidate =
    fallback.to_string();
  for param in header.split(';').skip(1) {
    let trimmed = param.trim();
    if trimmed.is_empty() {
      continue;
    }

    if let Some(value) = trimmed
      .strip_prefix("TYPE=")
      .or_else(|| {
        trimmed
          .strip_prefix("type=")
      })
    {
      for raw in value.split(',') {
        let token =
          raw.trim().to_ascii_lowercase();
        if token.is_empty() {
          continue;
        }
        let normalized =
          normalize_vcard_kind(
            &token, fallback,
          );
        if normalized != fallback {
          return normalized;
        }
        candidate = normalized;
      }
      continue;
    }

    if !trimmed.contains('=') {
      let normalized =
        normalize_vcard_kind(
        trimmed,
        fallback,
      );
      if normalized != fallback {
        return normalized;
      }
      candidate = normalized;
    }
  }

  candidate
}

fn parse_vcard_contacts(
  content: &str,
  source: &str,
) -> (Vec<ContactDto>, Vec<String>) {
  let mut unfolded = Vec::<String>::new();
  for line in content.lines() {
    if line.starts_with(' ')
      || line.starts_with('\t')
    {
      if let Some(last) =
        unfolded.last_mut()
      {
        last.push_str(line.trim());
      }
      continue;
    }
    unfolded.push(line.to_string());
  }

  let mut cards = Vec::<Vec<String>>::new();
  let mut current = Vec::<String>::new();
  let mut in_card = false;
  for line in unfolded {
    if line
      .trim()
      .eq_ignore_ascii_case(
        "BEGIN:VCARD"
      )
    {
      in_card = true;
      current.clear();
      continue;
    }
    if line
      .trim()
      .eq_ignore_ascii_case(
        "END:VCARD"
      )
    {
      if in_card {
        cards.push(current.clone());
      }
      in_card = false;
      current.clear();
      continue;
    }
    if in_card {
      current.push(line);
    }
  }

  let mut contacts =
    Vec::<ContactDto>::new();
  let mut errors = Vec::<String>::new();

  for (index, card) in
    cards.into_iter().enumerate()
  {
    let mut create = ContactCreate {
      display_name:  None,
      avatar_data_url: None,
      import_batch_id: None,
      source_file_name: None,
      given_name:    None,
      family_name:   None,
      nickname:      None,
      notes:         None,
      phones:        Vec::new(),
      emails:        Vec::new(),
      websites:      Vec::new(),
      birthday:      None,
      organization:  None,
      title:         None,
      addresses:     Vec::new(),
      source_id:     Some(
        format!(
          "import:{}",
          source
        ),
      ),
      source_kind:   Some(
        source.to_string(),
      ),
      remote_id:     None,
      link_group_id: None,
    };

    for line in card {
      let Some((header, raw_value)) =
        line.split_once(':')
      else {
        continue;
      };
      let key = header
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
      let value =
        raw_value.trim().to_string();

      match key.as_str() {
        | "FN" => {
          if !value.trim().is_empty() {
            create.display_name =
              Some(value);
          }
        }
        | "N" => {
          let mut parts =
            value.split(';');
          let family = parts
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
          let given = parts
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
          if !given.is_empty() {
            create.given_name =
              Some(given);
          }
          if !family.is_empty() {
            create.family_name =
              Some(family);
          }
        }
        | "NICKNAME" => {
          if !value.trim().is_empty() {
            create.nickname =
              Some(value);
          }
        }
        | "NOTE" => {
          if !value.trim().is_empty() {
            create.notes = Some(
              create
                .notes
                .take()
                .map(|old| {
                  format!(
                    "{old}\\n{value}"
                  )
                })
                .unwrap_or(value),
            );
          }
        }
        | "TEL" => {
          if !value.trim().is_empty() {
            create.phones.push(
              ContactFieldValue {
                value,
                kind: parse_vcard_type(
                  header,
                  "other",
                ),
                is_primary:
                  parse_vcard_primary(
                    header,
                  ),
              },
            );
          }
        }
        | "EMAIL" => {
          if !value.trim().is_empty() {
            create.emails.push(
              ContactFieldValue {
                value,
                kind: parse_vcard_type(
                  header,
                  "other",
                ),
                is_primary:
                  parse_vcard_primary(
                    header,
                  ),
              },
            );
          }
        }
        | "URL" => {
          if !value.trim().is_empty() {
            create
              .websites
              .push(
                ContactFieldValue {
                  value,
                  kind: parse_vcard_type(
                    header,
                    "website",
                  ),
                  is_primary:
                    parse_vcard_primary(
                      header,
                    ),
                },
              );
          }
        }
        | "PHOTO" => {
          // keep photo references in notes for deferred source-level processing.
          if !value.trim().is_empty()
            && !value
              .trim()
              .starts_with("data:")
          {
            let photo_ref = format!(
              "photo_ref:{}",
              value.trim()
            );
            create.notes = Some(
              create
                .notes
                .take()
                .map(|old| {
                  format!(
                    "{old}\\n{photo_ref}"
                  )
                })
                .unwrap_or(photo_ref),
            );
          }
        }
        | "BDAY" => {
          if !value.trim().is_empty() {
            create.birthday =
              Some(value);
          }
        }
        | "ORG" => {
          if !value.trim().is_empty() {
            create.organization =
              Some(value);
          }
        }
        | "TITLE" => {
          if !value.trim().is_empty() {
            create.title =
              Some(value);
          }
        }
        | _ => {}
      }
    }

    let mut contact =
      from_create_payload(
        create,
        None,
        None,
      );
    ensure_contact_defaults(
      &mut contact,
    );

    if let Err(error) =
      validate_contact(&contact)
    {
      errors.push(format!(
        "card {} skipped: {}",
        index + 1,
        error
      ));
      continue;
    }

    contacts.push(contact);
  }

  (contacts, errors)
}

fn import_source_kind(
  source: &str,
) -> String {
  let token =
    source.trim().to_ascii_lowercase();
  if token.contains("gmail")
    || token.contains("google")
  {
    return "gmail_file"
      .to_string();
  }
  if token.contains("iphone")
    || token.contains("icloud")
    || token.contains("apple")
  {
    return "iphone_file"
      .to_string();
  }
  "generic_vcard".to_string()
}

fn find_best_match(
  incoming: &ContactDto,
  existing: &[ContactDto],
) -> Option<(usize, u32, String)> {
  let mut best: Option<
    (usize, u32, String),
  > = None;
  for (index, item) in
    existing.iter().enumerate()
  {
    if let Some((score, reason)) =
      score_pair(incoming, item)
      && score >= 60
    {
      let replace = best
        .as_ref()
        .is_none_or(
          |(_, best_score, _)| {
            score > *best_score
          },
        );
      if replace {
        best = Some((
          index, score, reason,
        ));
      }
    }
  }
  best
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, query = ?args.query, limit = ?args.limit, cursor = ?args.cursor, source = ?args.source))]
pub async fn contacts_list(
  args: ContactsListArgs,
  request_id: Option<String>,
) -> Result<ContactsListResult, String> {
  info!(request_id = ?request_id, "contacts_list command invoked");
  let result =
    contacts_list_internal(args);
  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contacts_list command failed");
  }
  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, display_name = ?args.display_name, source = ?args.source_kind))]
pub async fn contact_add(
  args: ContactCreate,
  request_id: Option<String>,
) -> Result<ContactDto, String> {
  info!(request_id = ?request_id, "contact_add command invoked");

  let result = (|| -> anyhow::Result<
    ContactDto,
  > {
    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;
    let (
      contacts_path,
      _deleted,
      _batches,
      _undo,
    ) = ensure_contacts_store()?;

    let mut contacts =
      load_contacts_cached(
        &contacts_path,
      )?;

    let mut contact =
      from_create_payload(
        args,
        None,
        None,
      );
    ensure_contact_defaults(
      &mut contact,
    );
    validate_contact(&contact)?;

    contacts.push(contact.clone());
    sort_contacts(&mut contacts);
    save_jsonl(
      &contacts_path,
      &contacts,
    )?;
    set_contacts_cache(
      &contacts_path,
      &contacts,
    )?;

    Ok(contact)
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contact_add command failed");
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, id = %args.id))]
pub async fn contact_update(
  args: ContactUpdateArgs,
  request_id: Option<String>,
) -> Result<ContactDto, String> {
  info!(request_id = ?request_id, id = %args.id, "contact_update command invoked");

  let result = (|| -> anyhow::Result<
    ContactDto,
  > {
    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;
    let (
      contacts_path,
      _deleted,
      _batches,
      _undo,
    ) = ensure_contacts_store()?;

    let mut contacts =
      load_contacts_cached(
        &contacts_path,
      )?;

    let mut updated = None;
    for contact in &mut contacts {
      if contact.id == args.id {
        apply_contact_patch(
          contact,
          args.patch.clone(),
        );
        ensure_contact_defaults(
          contact,
        );
        validate_contact(contact)?;
        updated =
          Some(contact.clone());
        break;
      }
    }

    let Some(updated) = updated else {
      anyhow::bail!(
        "contact not found"
      );
    };

    sort_contacts(&mut contacts);
    save_jsonl(
      &contacts_path,
      &contacts,
    )?;
    set_contacts_cache(
      &contacts_path,
      &contacts,
    )?;

    Ok(updated)
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contact_update command failed");
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, id = %args.id))]
pub async fn contact_delete(
  args: ContactIdArg,
  request_id: Option<String>,
) -> Result<(), String> {
  info!(request_id = ?request_id, id = %args.id, "contact_delete command invoked");

  let result = (|| -> anyhow::Result<
    ()
  > {
    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;
    let (
      contacts_path,
      deleted_path,
      _batches,
      _undo,
    ) = ensure_contacts_store()?;

    let mut contacts =
      load_contacts_cached(
        &contacts_path,
      )?;

    let Some(index) = contacts
      .iter()
      .position(|contact| {
        contact.id == args.id
      })
    else {
      anyhow::bail!(
        "contact not found"
      );
    };

    let removed = contacts.remove(index);
    append_jsonl(
      &deleted_path,
      &removed,
    )?;
    save_jsonl(
      &contacts_path,
      &contacts,
    )?;
    set_contacts_cache(
      &contacts_path,
      &contacts,
    )?;

    Ok(())
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contact_delete command failed");
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, count = args.ids.len()))]
pub async fn contacts_delete_bulk(
  args: ContactsDeleteBulkArgs,
  request_id: Option<String>,
) -> Result<usize, String> {
  info!(request_id = ?request_id, count = args.ids.len(), "contacts_delete_bulk command invoked");

  let result = (|| -> anyhow::Result<
    usize
  > {
    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;
    let (
      contacts_path,
      deleted_path,
      _batches,
      _undo,
    ) = ensure_contacts_store()?;

    let contacts =
      load_contacts_cached(
        &contacts_path,
      )?;
    let ids = args
      .ids
      .into_iter()
      .collect::<HashSet<_>>();

    let mut kept = Vec::<ContactDto>::new();
    let mut removed = Vec::<ContactDto>::new();
    for contact in contacts {
      if ids.contains(&contact.id) {
        removed.push(contact);
      } else {
        kept.push(contact);
      }
    }

    for item in &removed {
      append_jsonl(
        &deleted_path,
        item,
      )?;
    }

    let deleted_count = removed.len();
    sort_contacts(&mut kept);
    save_jsonl(
      &contacts_path,
      &kept,
    )?;
    set_contacts_cache(
      &contacts_path,
      &kept,
    )?;

    Ok(deleted_count)
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contacts_delete_bulk command failed");
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, query = ?args.query))]
pub async fn contacts_dedupe_preview(
  args: ContactsDedupePreviewArgs,
  request_id: Option<String>,
) -> Result<
  ContactsDedupePreviewResult,
  String,
> {
  info!(request_id = ?request_id, "contacts_dedupe_preview command invoked");

  let result = (|| -> anyhow::Result<
    ContactsDedupePreviewResult,
  > {
    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;
    let (
      contacts_path,
      _deleted,
      _batches,
      _undo,
    ) = ensure_contacts_store()?;

    let contacts =
      load_contacts_cached(
        &contacts_path,
      )?;
    let decisions_path =
      contacts_dedupe_decisions_path(
        &contacts_path,
      )?;
    let decisions =
      load_dedupe_decisions_map(
        &decisions_path,
      )?;
    let groups = dedupe_groups(
      &contacts,
      args.query.as_deref(),
      &decisions,
    );

    Ok(
      ContactsDedupePreviewResult {
        groups,
      },
    )
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contacts_dedupe_preview command failed");
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, query = ?args.query))]
pub async fn contacts_dedupe_candidates(
  args: ContactsDedupePreviewArgs,
  request_id: Option<String>,
) -> Result<
  ContactsDedupePreviewResult,
  String,
> {
  contacts_dedupe_preview(
    args, request_id,
  )
  .await
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, group_id = %args.candidate_group_id, decision = %args.decision))]
pub async fn contacts_dedupe_decide(
  args: ContactsDedupeDecideArgs,
  request_id: Option<String>,
) -> Result<
  ContactsDedupeDecideResult,
  String,
> {
  info!(request_id = ?request_id, group_id = %args.candidate_group_id, decision = %args.decision, "contacts_dedupe_decide command invoked");

  let result = (|| -> anyhow::Result<
    ContactsDedupeDecideResult,
  > {
    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;
    let (
      contacts_path,
      _deleted,
      _batches,
      _undo,
    ) = ensure_contacts_store()?;

    let actor = args
      .actor
      .as_deref()
      .unwrap_or("user")
      .trim()
      .to_string();
    let decision = DedupDecision {
      candidate_group_id: args
        .candidate_group_id
        .trim()
        .to_string(),
      decision: args
        .decision
        .trim()
        .to_ascii_lowercase(),
      actor: if actor.is_empty() {
        "user".to_string()
      } else {
        actor
      },
      decided_at: now_iso(),
    };
    let decisions_path =
      contacts_dedupe_decisions_path(
        &contacts_path,
      )?;
    append_jsonl(
      &decisions_path,
      &decision,
    )?;

    Ok(ContactsDedupeDecideResult {
      candidate_group_id: decision
        .candidate_group_id,
      decision: decision.decision,
      actor: decision.actor,
      decided_at: decision.decided_at,
    })
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contacts_dedupe_decide command failed");
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, action = %args.action, id = %args.id))]
pub async fn contact_open_action(
  args: ContactOpenActionArgs,
  request_id: Option<String>,
) -> Result<ContactOpenActionResult, String> {
  info!(request_id = ?request_id, action = %args.action, id = %args.id, "contact_open_action command invoked");

  let result = (|| -> anyhow::Result<
    ContactOpenActionResult,
  > {
    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;
    let (
      contacts_path,
      _deleted,
      _batches,
      _undo,
    ) = ensure_contacts_store()?;

    let contacts =
      load_contacts_cached(
        &contacts_path,
      )?;
    let contact = contacts
      .into_iter()
      .find(|item| {
        item.id == args.id
      })
      .ok_or_else(|| {
        anyhow::anyhow!(
          "contact not found"
        )
      })?;

    let action = args
      .action
      .trim()
      .to_ascii_lowercase();
    let url = if action == "mailto"
      || action == "email"
    {
      let value = args
        .value
        .filter(|token| {
          !token.trim().is_empty()
        })
        .or_else(|| {
          contact
            .emails
            .iter()
            .find(|item| {
              item.is_primary
            })
            .or_else(|| {
              contact
                .emails
                .first()
            })
            .map(|item| {
              item.value.clone()
            })
        })
        .ok_or_else(|| {
          anyhow::anyhow!(
            "contact has no email"
          )
        })?;
      format!("mailto:{value}")
    } else if action == "tel"
      || action == "phone"
    {
      let value = args
        .value
        .filter(|token| {
          !token.trim().is_empty()
        })
        .or_else(|| {
          contact
            .phones
            .iter()
            .find(|item| {
              item.is_primary
            })
            .or_else(|| {
              contact
                .phones
                .first()
            })
            .map(|item| {
              item.value.clone()
            })
        })
        .ok_or_else(|| {
          anyhow::anyhow!(
            "contact has no phone"
          )
        })?;
      format!("tel:{value}")
    } else {
      anyhow::bail!(
        "unsupported action: {action}"
      );
    };

    Ok(ContactOpenActionResult {
      launched: false,
      url,
    })
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contact_open_action command failed");
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, source = %args.source, file_name = ?args.file_name))]
pub async fn contacts_import_preview(
  args: ContactsImportPreviewArgs,
  request_id: Option<String>,
) -> Result<
  ContactsImportPreviewResult,
  String,
> {
  info!(request_id = ?request_id, source = %args.source, "contacts_import_preview command invoked");

  let result = (|| -> anyhow::Result<
    ContactsImportPreviewResult,
  > {
    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;
    let (
      contacts_path,
      _deleted,
      _batches,
      _undo,
    ) = ensure_contacts_store()?;

    let existing =
      load_contacts_cached(
        &contacts_path,
      )?;

    let source_kind =
      import_source_kind(
        args.source.as_str(),
      );
    let batch_id =
      Uuid::new_v4().to_string();
    let (mut imported, errors) =
      parse_vcard_contacts(
        &args.content,
        &source_kind,
      );

    let mut conflicts =
      Vec::<ContactImportConflict>::new();
    for contact in &mut imported {
      contact.source_kind =
        source_kind.clone();
      contact.source_id = format!(
        "import:{}",
        source_kind
      );
      contact.import_batch_id =
        Some(batch_id.clone());
      contact.source_file_name =
        args.file_name.clone();

      if let Some((index, score, reason)) =
        find_best_match(
          contact, &existing,
        )
      {
        if score >= 80 {
          conflicts.push(
            ContactImportConflict {
              imported:
                contact.clone(),
              existing:
                existing[index]
                  .clone(),
              score,
              reason,
            },
          );
        }
      }
    }

    Ok(
      ContactsImportPreviewResult {
        batch_id,
        source: source_kind,
        total_rows: imported.len()
          + errors.len(),
        valid_rows: imported.len(),
        skipped_rows: errors.len(),
        potential_duplicates:
          conflicts.len(),
        contacts: imported,
        conflicts,
        errors,
      },
    )
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contacts_import_preview command failed");
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, source = %args.source, mode = %args.mode, file_name = ?args.file_name))]
pub async fn contacts_import_commit(
  args: ContactsImportCommitArgs,
  request_id: Option<String>,
) -> Result<
  ContactsImportCommitResult,
  String,
> {
  info!(request_id = ?request_id, source = %args.source, mode = %args.mode, "contacts_import_commit command invoked");

  let result = (|| -> anyhow::Result<
    ContactsImportCommitResult,
  > {
    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;

    let (
      contacts_path,
      _deleted,
      batches_path,
      _undo,
    ) = ensure_contacts_store()?;

    let mut existing =
      load_contacts_cached(
        &contacts_path,
      )?;

    let source_kind =
      import_source_kind(
        args.source.as_str(),
      );
    let batch_id =
      Uuid::new_v4().to_string();
    let (imported, errors) =
      parse_vcard_contacts(
        &args.content,
        &source_kind,
      );

    let mode = args
      .mode
      .trim()
      .to_ascii_lowercase();

    let mut created = 0_usize;
    let mut updated = 0_usize;
    let mut skipped = 0_usize;
    let mut conflicts = 0_usize;

    for mut incoming in imported {
      incoming.source_kind =
        source_kind.clone();
      incoming.source_id = format!(
        "import:{}",
        source_kind
      );
      incoming.import_batch_id =
        Some(batch_id.clone());
      incoming.source_file_name =
        args.file_name.clone();

      if let Some((index, score, _reason)) =
        find_best_match(
          &incoming,
          &existing,
        )
      {
        if score >= 80 {
          conflicts += 1;
          if mode == "upsert" {
            merge_contact_records(
              &mut existing[index],
              &incoming,
            );
            ensure_contact_defaults(
              &mut existing[index],
            );
            validate_contact(
              &existing[index],
            )?;
            updated += 1;
          } else if mode == "safe"
            || mode == "review"
          {
            skipped += 1;
          } else {
            skipped += 1;
          }
          continue;
        }
      }

      ensure_contact_defaults(
        &mut incoming,
      );
      validate_contact(&incoming)?;
      existing.push(incoming);
      created += 1;
    }

    sort_contacts(&mut existing);

    save_jsonl(
      &contacts_path,
      &existing,
    )?;
    set_contacts_cache(
      &contacts_path,
      &existing,
    )?;

    append_jsonl(
      &batches_path,
      &ContactImportBatch {
        id: batch_id.clone(),
        source_type: source_kind,
        file_name: args.file_name,
        imported_at: now_iso(),
        total_rows: created
          + updated
          + skipped
          + errors.len(),
        valid_rows: created + updated,
        skipped_rows: skipped
          + errors.len(),
      },
    )?;

    Ok(ContactsImportCommitResult {
      batch_id,
      created,
      updated,
      skipped,
      failed: errors.len(),
      conflicts,
      errors,
    })
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contacts_import_commit command failed");
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, count = args.ids.len(), target = ?args.target_id))]
pub async fn contacts_merge(
  args: ContactsMergeArgs,
  request_id: Option<String>,
) -> Result<ContactsMergeResult, String> {
  info!(request_id = ?request_id, count = args.ids.len(), target = ?args.target_id, "contacts_merge command invoked");

  let result = (|| -> anyhow::Result<
    ContactsMergeResult,
  > {
    if args.ids.len() < 2 {
      anyhow::bail!(
        "merge requires at least \
         two contacts"
      );
    }

    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;

    let (
      contacts_path,
      deleted_path,
      _batches,
      undo_path,
    ) = ensure_contacts_store()?;

    let mut contacts =
      load_contacts_cached(
        &contacts_path,
      )?;

    let before = contacts.clone();
    let ids = args
      .ids
      .into_iter()
      .collect::<BTreeSet<_>>();

    let selected = contacts
      .iter()
      .filter(|contact| {
        ids.contains(&contact.id)
      })
      .cloned()
      .collect::<Vec<_>>();

    if selected.len() < 2 {
      anyhow::bail!(
        "failed to find selected \
         contacts"
      );
    }

    let target_id = args
      .target_id
      .or_else(|| {
        selected
          .iter()
          .max_by(|left, right| {
            left.updated_at.cmp(
              &right.updated_at,
            )
          })
          .map(|contact| {
            contact.id
          })
      })
      .ok_or_else(|| {
        anyhow::anyhow!(
          "invalid merge target"
        )
      })?;

    let mut merged = selected
      .iter()
      .find(|contact| {
        contact.id == target_id
      })
      .cloned()
      .unwrap_or_else(|| {
        selected[0].clone()
      });

    let removed_ids = selected
      .iter()
      .map(|item| item.id)
      .filter(|id| {
        *id != merged.id
      })
      .collect::<Vec<_>>();
    let removed_contacts = selected
      .iter()
      .filter(|item| {
        item.id != merged.id
      })
      .cloned()
      .collect::<Vec<_>>();

    for contact in selected {
      if contact.id == merged.id {
        continue;
      }
      merge_contact_records(
        &mut merged,
        &contact,
      );
    }
    ensure_contact_defaults(
      &mut merged,
    );
    validate_contact(&merged)?;

    contacts.retain(|contact| {
      !ids.contains(&contact.id)
        || contact.id == merged.id
    });

    if let Some(slot) = contacts
      .iter_mut()
      .find(|contact| {
        contact.id == merged.id
      })
    {
      *slot = merged.clone();
    } else {
      contacts.push(merged.clone());
    }
    sort_contacts(&mut contacts);

    save_jsonl(
      &contacts_path,
      &contacts,
    )?;
    set_contacts_cache(
      &contacts_path,
      &contacts,
    )?;
    for removed in &removed_contacts {
      append_jsonl(
        &deleted_path,
        removed,
      )?;
    }

    let undo_id =
      Uuid::new_v4().to_string();
    append_jsonl(
      &undo_path,
      &ContactsMergeUndoEntry {
        undo_id: undo_id.clone(),
        contacts_before: before,
        created_at: now_iso(),
      },
    )?;
    let merge_audit_path =
      contacts_merge_audit_path(
        &contacts_path,
      )?;
    append_jsonl(
      &merge_audit_path,
      &MergeAudit {
        undo_id: undo_id.clone(),
        target_contact_id: merged.id,
        source_contact_ids:
          removed_ids.clone(),
        merge_payload:
          merged.clone(),
        operator:
          request_id.clone().unwrap_or_else(
            || "user".to_string(),
          ),
        created_at: now_iso(),
      },
    )?;
    let mut decision_group_ids =
      ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>();
    decision_group_ids.sort();
    let decisions_path =
      contacts_dedupe_decisions_path(
        &contacts_path,
      )?;
    append_jsonl(
      &decisions_path,
      &DedupDecision {
        candidate_group_id: format!(
          "group:{}",
          decision_group_ids.join(",")
        ),
        decision: "merged".to_string(),
        actor:
          request_id.clone().unwrap_or_else(
            || "user".to_string(),
          ),
        decided_at: now_iso(),
      },
    )?;

    Ok(ContactsMergeResult {
      merged,
      removed_ids,
      undo_id,
    })
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contacts_merge command failed");
  }

  result.map_err(err_to_string)
}

#[tauri::command]
#[instrument(fields(request_id = ?request_id, undo_id = ?args.undo_id))]
pub async fn contacts_merge_undo(
  args: ContactsMergeUndoArgs,
  request_id: Option<String>,
) -> Result<
  ContactsMergeUndoResult,
  String,
> {
  info!(request_id = ?request_id, undo_id = ?args.undo_id, "contacts_merge_undo command invoked");

  let result = (|| -> anyhow::Result<
    ContactsMergeUndoResult,
  > {
    let _guard = contacts_lock()
      .lock()
      .map_err(|_| {
        anyhow::anyhow!(
          "contacts store lock poisoned"
        )
      })?;

    let (
      contacts_path,
      _deleted,
      _batches,
      undo_path,
    ) = ensure_contacts_store()?;

    let mut entries = load_jsonl::<
      ContactsMergeUndoEntry,
    >(&undo_path)?;

    let selected_index = if let Some(
      requested,
    ) = args.undo_id.as_deref()
    {
      entries
        .iter()
        .position(|entry| {
          entry.undo_id
            == requested
        })
    } else if entries.is_empty() {
      None
    } else {
      Some(entries.len() - 1)
    };

    let Some(index) = selected_index
    else {
      anyhow::bail!(
        "no merge undo snapshot \
         available"
      );
    };

    let entry = entries.remove(index);
    save_jsonl(&undo_path, &entries)?;
    save_jsonl(
      &contacts_path,
      &entry.contacts_before,
    )?;
    set_contacts_cache(
      &contacts_path,
      &entry.contacts_before,
    )?;

    let merge_audit_path =
      contacts_merge_audit_path(
        &contacts_path,
      )?;
    let audits = load_jsonl::<
      MergeAudit,
    >(&merge_audit_path)?;
    if let Some(audit) = audits
      .iter()
      .rev()
      .find(|audit| {
        audit.undo_id
          == entry.undo_id
      })
    {
      let mut group_ids = vec![
        audit
          .target_contact_id
          .to_string(),
      ];
      for id in &audit.source_contact_ids {
        group_ids.push(id.to_string());
      }
      group_ids.sort();

      let decisions_path =
        contacts_dedupe_decisions_path(
          &contacts_path,
        )?;
      append_jsonl(
        &decisions_path,
        &DedupDecision {
          candidate_group_id: format!(
            "group:{}",
            group_ids.join(",")
          ),
          decision:
            "reopened".to_string(),
          actor: request_id
            .clone()
            .unwrap_or_else(|| {
              "user".to_string()
            }),
          decided_at: now_iso(),
        },
      )?;
    }

    Ok(ContactsMergeUndoResult {
      restored: entry
        .contacts_before
        .len(),
      undo_id: entry.undo_id,
    })
  })();

  if let Err(err) =
    result.as_ref()
  {
    error!(request_id = ?request_id, error = %err, "contacts_merge_undo command failed");
  }

  result.map_err(err_to_string)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::future::Future;
  use std::sync::{
    Mutex,
    OnceLock,
  };

  fn make_contact(
    display_name: &str,
    email: Option<&str>,
    phone: Option<&str>,
  ) -> ContactDto {
    from_create_payload(
      ContactCreate {
        display_name: Some(
          display_name.to_string(),
        ),
        avatar_data_url: None,
        import_batch_id: None,
        source_file_name: None,
        given_name: None,
        family_name: None,
        nickname: None,
        notes: None,
        phones: phone
          .map(|value| {
            vec![ContactFieldValue {
              value:
                value.to_string(),
              kind: "mobile"
                .to_string(),
              is_primary: true,
            }]
          })
          .unwrap_or_default(),
        emails: email
          .map(|value| {
            vec![ContactFieldValue {
              value:
                value.to_string(),
              kind: "home".to_string(),
              is_primary: true,
            }]
          })
          .unwrap_or_default(),
        websites: Vec::new(),
        birthday: None,
        organization: None,
        title: None,
        addresses: Vec::new(),
        source_id:
          Some("local".to_string()),
        source_kind:
          Some("local".to_string()),
        remote_id: None,
        link_group_id: None,
      },
      None,
      None,
    )
  }

  fn make_contact_create(
    display_name: &str,
    email: &str,
    phone: &str,
  ) -> ContactCreate {
    ContactCreate {
      display_name: Some(
        display_name.to_string(),
      ),
      avatar_data_url: None,
      import_batch_id: None,
      source_file_name: None,
      given_name: None,
      family_name: None,
      nickname: None,
      notes: None,
      phones: vec![ContactFieldValue {
        value: phone.to_string(),
        kind: "mobile".to_string(),
        is_primary: true,
      }],
      emails: vec![ContactFieldValue {
        value: email.to_string(),
        kind: "home".to_string(),
        is_primary: true,
      }],
      websites: Vec::new(),
      birthday: None,
      organization: None,
      title: None,
      addresses: Vec::new(),
      source_id:
        Some("local".to_string()),
      source_kind:
        Some("local".to_string()),
      remote_id: None,
      link_group_id: None,
    }
  }

  fn dedupe_pair_key(
    left: &str,
    right: &str,
  ) -> (String, String) {
    if left <= right {
      (
        left.to_string(),
        right.to_string(),
      )
    } else {
      (
        right.to_string(),
        left.to_string(),
      )
    }
  }

  fn dedupe_pairs(
    groups: &[
      ContactDedupeCandidateGroup
    ]
  ) -> HashSet<(String, String)> {
    let mut pairs = HashSet::<(
      String,
      String,
    )>::new();
    for group in groups {
      for left in 0
        ..group.contacts.len()
      {
        for right in
          (left + 1)
            ..group.contacts.len()
        {
          let left_id = group.contacts[left]
            .id
            .to_string();
          let right_id =
            group.contacts[right]
              .id
              .to_string();
          pairs.insert(dedupe_pair_key(
            &left_id, &right_id,
          ));
        }
      }
    }
    pairs
  }

  fn run_async<T>(
    future: impl Future<
      Output = T,
    >,
  ) -> T {
    tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .expect("tokio runtime")
      .block_on(future)
  }

  fn temp_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> =
      OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
  }

  fn with_temp_contacts_dir(
    run: impl FnOnce(),
  ) {
    let _guard = temp_env_lock()
      .lock()
      .expect("temp env lock");
    let previous = std::env::var(
      "RIVET_GUI_DATA",
    )
    .ok();
    let temp_dir = std::env::temp_dir()
      .join(format!(
        "rivet_contacts_test_{}",
        Uuid::new_v4()
      ));
    std::fs::create_dir_all(&temp_dir)
      .expect("create temp dir");
    unsafe {
      std::env::set_var(
        "RIVET_GUI_DATA",
        &temp_dir,
      );
    }
    if let Ok(mut cache) =
      contacts_index_cache().lock()
    {
      *cache =
        ContactsIndexCache::default();
    }

    run();

    if let Some(previous) = previous {
      unsafe {
        std::env::set_var(
          "RIVET_GUI_DATA",
          previous,
        );
      }
    } else {
      unsafe {
        std::env::remove_var(
          "RIVET_GUI_DATA",
        );
      }
    }
    if let Ok(mut cache) =
      contacts_index_cache().lock()
    {
      *cache =
        ContactsIndexCache::default();
    }
    let _ = std::fs::remove_dir_all(
      &temp_dir,
    );
  }

  #[test]
  fn normalize_text_is_diacritic_insensitive()
  {
    assert_eq!(
      normalize_text("  José Núñez  "),
      "jose nunez"
    );
    assert_eq!(
      normalize_text("MÜNCHEN"),
      "munchen"
    );
  }

  #[test]
  fn validate_contact_enforces_identity_and_limits()
  {
    let mut contact = make_contact(
      "",
      None,
      None,
    );
    assert!(
      validate_contact(&contact)
        .is_err()
    );

    contact.display_name = "A".repeat(
      CONTACTS_MAX_NAME_LEN + 1
    );
    assert!(
      validate_contact(&contact)
        .is_err()
    );
  }

  #[test]
  fn score_pair_uses_email_or_phone_strong_signal()
  {
    let left = make_contact(
      "Alice",
      Some("alice@example.com"),
      None,
    );
    let right = make_contact(
      "Alice B",
      Some("alice@example.com"),
      None,
    );
    let score =
      score_pair(&left, &right)
        .expect("pair score")
        .0;
    assert_eq!(score, 100);
  }

  #[test]
  fn dedupe_groups_respects_prior_decisions()
  {
    let left = make_contact(
      "Chris",
      Some("c@example.com"),
      None,
    );
    let right = make_contact(
      "Chris",
      Some("c@example.com"),
      None,
    );
    let ids = {
      let mut tokens = vec![
        left.id.to_string(),
        right.id.to_string(),
      ];
      tokens.sort();
      format!(
        "group:{}",
        tokens.join(",")
      )
    };
    let mut decisions = HashMap::<
      String,
      DedupDecision,
    >::new();
    decisions.insert(
      ids.clone(),
      DedupDecision {
        candidate_group_id: ids,
        decision: "ignored"
          .to_string(),
        actor: "test".to_string(),
        decided_at: now_iso(),
      },
    );

    let groups = dedupe_groups(
      &[left, right],
      None,
      &decisions,
    );
    assert!(groups.is_empty());
  }

  #[test]
  fn matching_accuracy_stays_above_thresholds()
  {
    let mut c1 = make_contact(
      "Alice Johnson",
      Some("alice@acme.com"),
      None,
    );
    c1.organization =
      Some("Acme".to_string());
    let mut c2 = make_contact(
      "Alice Johnson",
      Some("alice@acme.com"),
      None,
    );
    c2.organization =
      Some("Acme".to_string());

    let c3 = make_contact(
      "Bob Chen",
      None,
      Some("+1 (555) 111-1111"),
    );
    let c4 = make_contact(
      "Robert Chen",
      None,
      Some("+1 555 111 1111"),
    );

    let mut c5 = make_contact(
      "Carla Mendez",
      Some("carla@contoso.com"),
      None,
    );
    c5.organization = Some(
      "Contoso".to_string(),
    );
    let mut c6 = make_contact(
      "Karla Mendez",
      Some("karla@contoso.com"),
      None,
    );
    c6.organization = Some(
      "Contoso".to_string(),
    );

    let c7 = make_contact(
      "Daniel Lee",
      Some("daniel@alpha.com"),
      None,
    );
    let c8 = make_contact(
      "Dana Lee",
      Some("dana@beta.com"),
      None,
    );
    let c9 = make_contact(
      "Eve Adams",
      Some("eve@random.com"),
      None,
    );
    let c10 = make_contact(
      "Evan Adamson",
      Some("evan@elsewhere.com"),
      None,
    );

    let contacts = vec![
      c1.clone(),
      c2.clone(),
      c3.clone(),
      c4.clone(),
      c5.clone(),
      c6.clone(),
      c7, c8, c9, c10,
    ];
    let decisions = HashMap::<
      String,
      DedupDecision,
    >::new();
    let groups = dedupe_groups(
      &contacts,
      None,
      &decisions,
    );
    let predicted = dedupe_pairs(
      &groups,
    );

    let expected = HashSet::from([
      dedupe_pair_key(
        &c1.id.to_string(),
        &c2.id.to_string(),
      ),
      dedupe_pair_key(
        &c3.id.to_string(),
        &c4.id.to_string(),
      ),
      dedupe_pair_key(
        &c5.id.to_string(),
        &c6.id.to_string(),
      ),
    ]);

    let true_positives = predicted
      .iter()
      .filter(|pair| {
        expected.contains(*pair)
      })
      .count();
    let precision =
      if predicted.is_empty() {
        0.0
      } else {
        true_positives as f32
          / predicted.len() as f32
      };
    let recall = true_positives
      as f32
      / expected.len() as f32;

    assert!(
      precision >= 0.85,
      "precision below threshold: \
       {precision}"
    );
    assert!(
      recall >= 0.85,
      "recall below threshold: \
       {recall}"
    );
  }

  #[test]
  fn merge_contact_records_unions_unique_fields()
  {
    let mut target = make_contact(
      "Dana",
      Some("dana@one.test"),
      Some("+1 555 000"),
    );
    target.notes = None;
    let mut source = make_contact(
      "Dana",
      Some("dana@two.test"),
      Some("+1 555 111"),
    );
    source.notes =
      Some("new note".to_string());

    merge_contact_records(
      &mut target,
      &source,
    );

    assert_eq!(
      target.emails.len(),
      2
    );
    assert_eq!(
      target.phones.len(),
      2
    );
    assert_eq!(
      target.notes.as_deref(),
      Some("new note")
    );
  }

  #[test]
  fn merge_and_undo_preserve_audit_and_dedupe_reopen()
  {
    with_temp_contacts_dir(|| {
      let left = run_async(
        contact_add(
          make_contact_create(
            "Morgan Lane",
            "morgan.one@example.com",
            "+1 555 0100",
          ),
          None,
        ),
      )
      .expect("add left");
      let right = run_async(
        contact_add(
          make_contact_create(
            "Morgan Lane",
            "morgan.two@example.com",
            "+1 555 0100",
          ),
          None,
        ),
      )
      .expect("add right");

      let dedupe_before = run_async(
        contacts_dedupe_preview(
          ContactsDedupePreviewArgs {
            query: None,
          },
          None,
        ),
      )
      .expect("dedupe preview");
      assert_eq!(
        dedupe_before.groups.len(),
        1
      );

      let merge = run_async(
        contacts_merge(
          ContactsMergeArgs {
            ids: vec![
              left.id,
              right.id,
            ],
            target_id: Some(left.id),
          },
          Some("tester".to_string()),
        ),
      )
      .expect("merge");
      assert_eq!(merge.merged.id, left.id);
      assert_eq!(
        merge.removed_ids,
        vec![right.id]
      );
      assert_eq!(
        merge.merged.emails.len(),
        2
      );
      assert_eq!(
        merge
          .merged
          .emails
          .iter()
          .filter(|item| {
            item.is_primary
          })
          .count(),
        1
      );

      let (
        contacts_path,
        _deleted,
        _batches,
        _undo_path,
      ) = ensure_contacts_store()
        .expect("store");
      let audit_path =
        contacts_merge_audit_path(
          &contacts_path,
        )
        .expect("audit path");
      let audits = load_jsonl::<
        MergeAudit,
      >(&audit_path)
      .expect("load audits");
      let audit = audits
        .iter()
        .find(|entry| {
          entry.undo_id
            == merge.undo_id
        })
        .expect("merge audit");
      assert_eq!(
        audit.target_contact_id,
        merge.merged.id
      );
      assert_eq!(
        audit.source_contact_ids,
        vec![right.id]
      );
      assert_eq!(
        audit.operator, "tester"
      );

      let dedupe_after_merge =
        run_async(
          contacts_dedupe_preview(
            ContactsDedupePreviewArgs {
              query: None,
            },
            None,
          ),
        )
        .expect(
          "dedupe after merge"
        );
      assert!(
        dedupe_after_merge.groups.is_empty()
      );

      let undo = run_async(
        contacts_merge_undo(
          ContactsMergeUndoArgs {
            undo_id: Some(
              merge.undo_id.clone(),
            ),
          },
          Some("tester".to_string()),
        ),
      )
      .expect("merge undo");
      assert_eq!(
        undo.undo_id,
        merge.undo_id
      );
      assert_eq!(undo.restored, 2);

      let listed = run_async(
        contacts_list(
          ContactsListArgs {
            query: None,
            limit: Some(200),
            cursor: None,
            source: None,
            updated_after: None,
          },
          None,
        ),
      )
      .expect("contacts list");
      assert_eq!(listed.total, 2);

      let dedupe_after_undo =
        run_async(
          contacts_dedupe_preview(
            ContactsDedupePreviewArgs {
              query: None,
            },
            None,
          ),
        )
        .expect(
          "dedupe after undo"
        );
      assert_eq!(
        dedupe_after_undo
          .groups
          .len(),
        1
      );

      let decisions_path =
        contacts_dedupe_decisions_path(
          &contacts_path,
        )
        .expect("decisions path");
      let decisions = load_jsonl::<
        DedupDecision,
      >(&decisions_path)
      .expect("load decisions");
      assert!(decisions.iter().any(
        |decision| {
          decision.decision
            == "merged"
        }
      ));
      assert!(decisions.iter().any(
        |decision| {
          decision.decision
            == "reopened"
        }
      ));
    });
  }

  #[test]
  fn parse_vcard_preserves_primary_flags()
  {
    let payload = concat!(
      "BEGIN:VCARD\n",
      "VERSION:3.0\n",
      "FN:Example Person\n",
      "EMAIL;TYPE=HOME,PREF:person@example.com\n",
      "TEL;TYPE=CELL,PREF:+1-555-222\n",
      "END:VCARD\n"
    );
    let (contacts, errors) =
      parse_vcard_contacts(
        payload,
        "gmail_file",
      );
    assert!(errors.is_empty());
    assert_eq!(contacts.len(), 1);
    assert!(
      contacts[0]
        .emails
        .first()
        .is_some_and(|item| {
          item.is_primary
        })
    );
    assert!(
      contacts[0]
        .phones
        .first()
        .is_some_and(|item| {
          item.is_primary
        })
    );
  }

  #[test]
  fn parses_gmail_fixture_with_expected_labels()
  {
    let payload = include_str!(
      "fixtures/contacts_gmail_sample.vcf"
    );
    let (contacts, errors) =
      parse_vcard_contacts(
        payload,
        "gmail_file",
      );
    assert!(errors.is_empty());
    assert_eq!(contacts.len(), 1);
    assert_eq!(
      contacts[0]
        .emails
        .first()
        .map(|item| item.kind.as_str()),
      Some("home")
    );
  }

  #[test]
  fn parses_iphone_fixture_and_keeps_photo_reference()
  {
    let payload = include_str!(
      "fixtures/contacts_iphone_sample.vcf"
    );
    let (contacts, errors) =
      parse_vcard_contacts(
        payload,
        "iphone_file",
      );
    assert!(errors.is_empty());
    assert_eq!(contacts.len(), 1);
    let notes = contacts[0]
      .notes
      .as_deref()
      .unwrap_or_default();
    assert!(
      notes.contains(
        "photo_ref:https://example.invalid/photo/taylor.jpg"
      )
    );
  }
}
