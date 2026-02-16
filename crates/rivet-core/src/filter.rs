use chrono::{DateTime, Utc};
use tracing::trace;

use crate::datetime::parse_date_expr;
use crate::task::{Status, Task};

#[derive(Debug, Clone)]
pub enum Pred {
    Id(u64),
    Uuid(uuid::Uuid),
    TagInclude(String),
    TagExclude(String),
    ProjectEq(String),
    StatusEq(Status),
    Waiting,
    DueBefore(DateTime<Utc>),
    DueAfter(DateTime<Utc>),
    TextContains(String),
}

#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub preds: Vec<Pred>,
}

impl Filter {
    #[tracing::instrument(skip(terms, now))]
    pub fn parse(terms: &[String], now: DateTime<Utc>) -> anyhow::Result<Self> {
        let mut preds = Vec::new();

        for term in terms {
            if let Some(tag) = term.strip_prefix('+') {
                preds.push(Pred::TagInclude(tag.to_string()));
                continue;
            }
            if let Some(tag) = term.strip_prefix('-') {
                preds.push(Pred::TagExclude(tag.to_string()));
                continue;
            }
            if let Ok(id) = term.parse::<u64>() {
                preds.push(Pred::Id(id));
                continue;
            }
            if let Ok(uuid) = uuid::Uuid::parse_str(term) {
                preds.push(Pred::Uuid(uuid));
                continue;
            }

            if let Some(project) = term.strip_prefix("project:") {
                preds.push(Pred::ProjectEq(project.to_string()));
                continue;
            }

            if let Some(status_text) = term.strip_prefix("status:") {
                match status_text.to_ascii_lowercase().as_str() {
                    "pending" => preds.push(Pred::StatusEq(Status::Pending)),
                    "completed" => preds.push(Pred::StatusEq(Status::Completed)),
                    "deleted" => preds.push(Pred::StatusEq(Status::Deleted)),
                    "waiting" => preds.push(Pred::Waiting),
                    _ => preds.push(Pred::TextContains(term.clone())),
                }
                continue;
            }

            if let Some(value) = term.strip_prefix("due.before:") {
                preds.push(Pred::DueBefore(parse_date_expr(value, now)?));
                continue;
            }

            if let Some(value) = term.strip_prefix("due.after:") {
                preds.push(Pred::DueAfter(parse_date_expr(value, now)?));
                continue;
            }

            preds.push(Pred::TextContains(term.clone()));
        }

        Ok(Self { preds })
    }

    #[tracing::instrument(skip(self, task, now))]
    pub fn matches(&self, task: &Task, now: DateTime<Utc>) -> bool {
        for pred in &self.preds {
            let ok = match pred {
                Pred::Id(id) => task.id == Some(*id),
                Pred::Uuid(uuid) => task.uuid == *uuid,
                Pred::TagInclude(tag) => task.tags.iter().any(|t| t == tag),
                Pred::TagExclude(tag) => task.tags.iter().all(|t| t != tag),
                Pred::ProjectEq(project) => task.project.as_deref() == Some(project.as_str()),
                Pred::StatusEq(status) => match status {
                    Status::Pending => task.status == Status::Pending && !task.is_waiting(now),
                    _ => &task.status == status,
                },
                Pred::Waiting => task.is_waiting(now),
                Pred::DueBefore(dt) => task.due.map(|due| due < *dt).unwrap_or(false),
                Pred::DueAfter(dt) => task.due.map(|due| due > *dt).unwrap_or(false),
                Pred::TextContains(text) => task
                    .description
                    .to_ascii_lowercase()
                    .contains(&text.to_ascii_lowercase()),
            };

            trace!(pred = ?pred, id = ?task.id, uuid = %task.uuid, ok, "filter predicate evaluation");
            if !ok {
                return false;
            }
        }

        if task.is_waiting(now) && !self.has_explicit_status_filter() {
            return false;
        }

        true
    }

    fn has_explicit_status_filter(&self) -> bool {
        self.preds
            .iter()
            .any(|pred| matches!(pred, Pred::StatusEq(_) | Pred::Waiting))
    }
}
