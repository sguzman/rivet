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
  ContactIdArg,
  ContactImportBatch,
  ContactImportConflict,
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

  for path in [
    &contacts,
    &deleted,
    &batches,
    &merge_undo,
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

  Ok((
    contacts, deleted, batches,
    merge_undo,
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

fn normalize_text(
  value: &str
) -> String {
  value
    .trim()
    .to_ascii_lowercase()
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
    if group.len() == 1 {
      group[0].is_primary = true;
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

  haystacks
    .into_iter()
    .any(|token| token.contains(&q))
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

  let mut contacts =
    load_jsonl::<ContactDto>(
      &contacts_path
    )?;
  for contact in &mut contacts {
    ensure_contact_defaults(contact);
  }

  let updated_after =
    parse_updated_after(
      args.updated_after.as_deref(),
    )?;

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

    if let Some(query) =
      args.query.as_deref()
    {
      return contact_matches_query(
        contact, query,
      );
    }

    true
  });

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

  let total = contacts.len();
  let limit = args
    .limit
    .unwrap_or(200)
    .clamp(1, 500);
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

fn dedupe_groups(
  contacts: &[ContactDto],
  query: Option<&str>,
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

  for (root, members) in grouped {
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

    out.push(
      ContactDedupeCandidateGroup {
        group_id: format!(
          "group-{root}"
        ),
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
      target.push(field.clone());
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

fn parse_vcard_type(
  header: &str,
  fallback: &str,
) -> String {
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
      let token = value
        .split(',')
        .next()
        .unwrap_or_default()
        .trim();
      if !token.is_empty() {
        return token
          .to_ascii_lowercase();
      }
    }

    if !trimmed.contains('=') {
      return trimmed
        .to_ascii_lowercase();
    }
  }

  fallback.to_string()
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
                is_primary: false,
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
                is_primary: false,
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
                  is_primary: false,
                },
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
      load_jsonl::<ContactDto>(
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
    contacts.sort_by(|a, b| {
      normalize_text(&a.display_name)
        .cmp(&normalize_text(
          &b.display_name,
        ))
    });
    save_jsonl(
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
      load_jsonl::<ContactDto>(
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

    save_jsonl(
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
      load_jsonl::<ContactDto>(
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
      load_jsonl::<ContactDto>(
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
    save_jsonl(
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
      load_jsonl::<ContactDto>(
        &contacts_path,
      )?;
    let groups = dedupe_groups(
      &contacts,
      args.query.as_deref(),
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
      load_jsonl::<ContactDto>(
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
      load_jsonl::<ContactDto>(
        &contacts_path,
      )?;

    let source_kind =
      import_source_kind(
        args.source.as_str(),
      );
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

    let batch_id =
      Uuid::new_v4().to_string();

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
      load_jsonl::<ContactDto>(
        &contacts_path,
      )?;

    let source_kind =
      import_source_kind(
        args.source.as_str(),
      );
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

    existing.sort_by(|a, b| {
      normalize_text(&a.display_name)
        .cmp(&normalize_text(
          &b.display_name,
        ))
    });

    save_jsonl(
      &contacts_path,
      &existing,
    )?;

    let batch_id =
      Uuid::new_v4().to_string();
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
      _deleted,
      _batches,
      undo_path,
    ) = ensure_contacts_store()?;

    let mut contacts =
      load_jsonl::<ContactDto>(
        &contacts_path,
      )?;

    let before = contacts.clone();
    let ids = args
      .ids
      .into_iter()
      .collect::<BTreeSet<_>>();

    let target_id = args
      .target_id
      .or_else(|| {
        ids.iter().next().copied()
      })
      .ok_or_else(|| {
        anyhow::anyhow!(
          "invalid merge target"
        )
      })?;

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

    save_jsonl(
      &contacts_path,
      &contacts,
    )?;

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
