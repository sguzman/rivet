use serde::{
  Deserialize,
  Serialize
};
use uuid::Uuid;

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
)]
pub enum TaskStatus {
  Pending,
  Completed,
  Deleted,
  Waiting
}

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
)]
pub enum TaskPriority {
  Low,
  Medium,
  High
}

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
)]
pub struct TaskDto {
  pub uuid:        Uuid,
  pub id:          Option<u64>,
  #[serde(default)]
  pub title:       String,
  #[serde(default)]
  pub description: String,
  pub status:      TaskStatus,
  pub project:     Option<String>,
  pub tags:        Vec<String>,
  pub priority:    Option<TaskPriority>,
  pub due:         Option<String>,
  pub wait:        Option<String>,
  pub scheduled:   Option<String>,
  pub created:     Option<String>,
  pub modified:    Option<String>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct TasksListArgs {
  pub query:   Option<String>,
  pub status:  Option<TaskStatus>,
  pub project: Option<String>,
  pub tag:     Option<String>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct TaskCreate {
  pub title:       String,
  pub description: String,
  pub project:     Option<String>,
  pub tags:        Vec<String>,
  pub priority:    Option<TaskPriority>,
  pub due:         Option<String>,
  pub wait:        Option<String>,
  pub scheduled:   Option<String>
}

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  Default,
)]
pub struct TaskPatch {
  pub title:       Option<String>,
  pub description: Option<String>,
  pub project: Option<Option<String>>,
  pub tags:        Option<Vec<String>>,
  pub priority:
    Option<Option<TaskPriority>>,
  pub due: Option<Option<String>>,
  pub wait: Option<Option<String>>,
  pub scheduled: Option<Option<String>>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct TaskIdArg {
  pub uuid: Uuid
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct TaskUpdateArgs {
  pub uuid:  Uuid,
  pub patch: TaskPatch
}

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
)]
pub struct ContactFieldValue {
  pub value:      String,
  pub kind:       String,
  #[serde(default)]
  pub is_primary: bool
}

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
)]
pub struct ContactAddress {
  #[serde(default)]
  pub kind:        String,
  #[serde(default)]
  pub street:      String,
  #[serde(default)]
  pub city:        String,
  #[serde(default)]
  pub region:      String,
  #[serde(default)]
  pub postal_code: String,
  #[serde(default)]
  pub country:     String
}

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  PartialEq,
  Eq,
)]
pub struct ContactDto {
  pub id:            Uuid,
  #[serde(default)]
  pub display_name:  String,
  pub avatar_data_url: Option<String>,
  pub given_name:    Option<String>,
  pub family_name:   Option<String>,
  pub nickname:      Option<String>,
  pub notes:         Option<String>,
  #[serde(default)]
  pub phones: Vec<ContactFieldValue>,
  #[serde(default)]
  pub emails: Vec<ContactFieldValue>,
  #[serde(default)]
  pub websites: Vec<ContactFieldValue>,
  pub birthday:      Option<String>,
  pub organization:  Option<String>,
  pub title:         Option<String>,
  #[serde(default)]
  pub addresses: Vec<ContactAddress>,
  #[serde(default)]
  pub source_id:     String,
  #[serde(default)]
  pub source_kind:   String,
  pub remote_id:     Option<String>,
  pub link_group_id: Option<String>,
  pub created_at:    String,
  pub updated_at:    String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsListArgs {
  pub query:         Option<String>,
  pub limit:         Option<usize>,
  pub cursor:        Option<String>,
  pub source:        Option<String>,
  pub updated_after: Option<String>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsListResult {
  pub contacts:    Vec<ContactDto>,
  pub next_cursor: Option<String>,
  pub total:       usize
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactCreate {
  pub display_name:  Option<String>,
  pub avatar_data_url: Option<String>,
  pub given_name:    Option<String>,
  pub family_name:   Option<String>,
  pub nickname:      Option<String>,
  pub notes:         Option<String>,
  #[serde(default)]
  pub phones: Vec<ContactFieldValue>,
  #[serde(default)]
  pub emails: Vec<ContactFieldValue>,
  #[serde(default)]
  pub websites: Vec<ContactFieldValue>,
  pub birthday:      Option<String>,
  pub organization:  Option<String>,
  pub title:         Option<String>,
  #[serde(default)]
  pub addresses: Vec<ContactAddress>,
  pub source_id:     Option<String>,
  pub source_kind:   Option<String>,
  pub remote_id:     Option<String>,
  pub link_group_id: Option<String>
}

#[derive(
  Debug,
  Clone,
  Serialize,
  Deserialize,
  Default,
)]
pub struct ContactPatch {
  pub display_name:
    Option<Option<String>>,
  pub avatar_data_url:
    Option<Option<String>>,
  pub given_name:
    Option<Option<String>>,
  pub family_name:
    Option<Option<String>>,
  pub nickname: Option<Option<String>>,
  pub notes: Option<Option<String>>,
  pub phones:
    Option<Vec<ContactFieldValue>>,
  pub emails:
    Option<Vec<ContactFieldValue>>,
  pub websites:
    Option<Vec<ContactFieldValue>>,
  pub birthday: Option<Option<String>>,
  pub organization:
    Option<Option<String>>,
  pub title: Option<Option<String>>,
  pub addresses:
    Option<Vec<ContactAddress>>,
  pub source_id: Option<Option<String>>,
  pub source_kind:
    Option<Option<String>>,
  pub remote_id: Option<Option<String>>,
  pub link_group_id:
    Option<Option<String>>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactUpdateArgs {
  pub id:    Uuid,
  pub patch: ContactPatch
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactIdArg {
  pub id: Uuid
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsDeleteBulkArgs {
  pub ids: Vec<Uuid>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactOpenActionArgs {
  pub id:     Uuid,
  pub action: String,
  pub value:  Option<String>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactOpenActionResult {
  pub launched: bool,
  pub url:      String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsDedupePreviewArgs {
  pub query: Option<String>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactDedupeCandidateGroup {
  pub group_id: String,
  pub reason:   String,
  pub score:    u32,
  pub contacts: Vec<ContactDto>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsDedupePreviewResult {
  pub groups:
    Vec<ContactDedupeCandidateGroup>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactImportConflict {
  pub imported: ContactDto,
  pub existing: ContactDto,
  pub score:    u32,
  pub reason:   String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsImportPreviewArgs {
  pub source:    String,
  pub file_name: Option<String>,
  pub content:   String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsImportPreviewResult {
  pub batch_id:             String,
  pub source:               String,
  pub total_rows:           usize,
  pub valid_rows:           usize,
  pub skipped_rows:         usize,
  pub potential_duplicates: usize,
  pub contacts: Vec<ContactDto>,
  pub conflicts:
    Vec<ContactImportConflict>,
  pub errors:               Vec<String>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsImportCommitArgs {
  pub source:    String,
  pub file_name: Option<String>,
  pub content:   String,
  pub mode:      String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsImportCommitResult {
  pub batch_id:  String,
  pub created:   usize,
  pub updated:   usize,
  pub skipped:   usize,
  pub failed:    usize,
  pub conflicts: usize,
  pub errors:    Vec<String>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsMergeArgs {
  pub ids:       Vec<Uuid>,
  pub target_id: Option<Uuid>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsMergeResult {
  pub merged:      ContactDto,
  pub removed_ids: Vec<Uuid>,
  pub undo_id:     String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsMergeUndoArgs {
  pub undo_id: Option<String>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactsMergeUndoResult {
  pub restored: usize,
  pub undo_id:  String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactImportBatch {
  pub id:           String,
  pub source_type:  String,
  pub file_name:    Option<String>,
  pub imported_at:  String,
  pub total_rows:   usize,
  pub valid_rows:   usize,
  pub skipped_rows: usize
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct ContactIdentityFingerprint {
  pub contact_id:    Uuid,
  pub name_key:      String,
  pub primary_email: Option<String>,
  pub primary_phone: Option<String>
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct MergeAudit {
  pub undo_id:            String,
  pub target_contact_id:  Uuid,
  pub source_contact_ids: Vec<Uuid>,
  pub created_at:         String
}

#[derive(
  Debug, Clone, Serialize, Deserialize,
)]
pub struct DedupDecision {
  pub candidate_group_id: String,
  pub decision:           String,
  pub actor:              String,
  pub decided_at:         String
}
