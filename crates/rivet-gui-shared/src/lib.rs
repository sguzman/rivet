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
