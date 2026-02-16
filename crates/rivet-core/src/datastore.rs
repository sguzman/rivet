use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use tempfile::NamedTempFile;
use tracing::{debug, info};
use uuid::Uuid;

use crate::task::{Status, Task};

#[derive(Debug)]
pub struct DataStore {
    pub data_dir: PathBuf,
    pub pending_path: PathBuf,
    pub completed_path: PathBuf,
}

impl DataStore {
    #[tracing::instrument(skip(data_dir))]
    pub fn open(data_dir: &Path) -> anyhow::Result<Self> {
        let data_dir = data_dir.to_path_buf();
        fs::create_dir_all(&data_dir)
            .with_context(|| format!("failed to create {}", data_dir.display()))?;

        let pending_path = data_dir.join("pending.data");
        let completed_path = data_dir.join("completed.data");

        if !pending_path.exists() {
            fs::write(&pending_path, "")?;
        }
        if !completed_path.exists() {
            fs::write(&completed_path, "")?;
        }

        info!(
            data_dir = %data_dir.display(),
            pending = %pending_path.display(),
            completed = %completed_path.display(),
            "opened datastore"
        );

        Ok(Self {
            data_dir,
            pending_path,
            completed_path,
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn load_pending(&self) -> anyhow::Result<Vec<Task>> {
        load_jsonl(&self.pending_path).context("failed to load pending.data")
    }

    #[tracing::instrument(skip(self))]
    pub fn load_completed(&self) -> anyhow::Result<Vec<Task>> {
        load_jsonl(&self.completed_path).context("failed to load completed.data")
    }

    #[tracing::instrument(skip(self, tasks))]
    pub fn save_pending(&self, tasks: &[Task]) -> anyhow::Result<()> {
        save_jsonl_atomic(&self.pending_path, tasks).context("failed to save pending.data")
    }

    #[tracing::instrument(skip(self, tasks))]
    pub fn save_completed(&self, tasks: &[Task]) -> anyhow::Result<()> {
        save_jsonl_atomic(&self.completed_path, tasks).context("failed to save completed.data")
    }

    pub fn next_id(&self, pending: &[Task]) -> u64 {
        pending.iter().filter_map(|t| t.id).max().unwrap_or(0) + 1
    }

    #[tracing::instrument(skip(self, pending, task), fields(id = ?task.id, uuid = %task.uuid))]
    pub fn add_task(&self, mut pending: Vec<Task>, task: Task) -> anyhow::Result<Vec<Task>> {
        pending.push(task);
        pending.sort_by_key(|t| t.id.unwrap_or(u64::MAX));
        self.save_pending(&pending)?;
        Ok(pending)
    }

    #[tracing::instrument(skip(self), fields(uuid = %uuid))]
    pub fn move_to_completed(&self, uuid: Uuid) -> anyhow::Result<()> {
        let mut pending = self.load_pending()?;
        let mut completed = self.load_completed()?;

        let idx = pending
            .iter()
            .position(|t| t.uuid == uuid)
            .ok_or_else(|| anyhow!("task not found in pending: {uuid}"))?;

        let task = pending.remove(idx);
        completed.push(task);

        pending.sort_by_key(|t| t.id.unwrap_or(u64::MAX));
        completed.sort_by_key(|t| t.end);

        self.save_pending(&pending)?;
        self.save_completed(&completed)?;
        Ok(())
    }

    #[tracing::instrument(skip(self, tasks))]
    pub fn update_pending(&self, tasks: &[Task]) -> anyhow::Result<()> {
        self.save_pending(tasks)
    }

    #[tracing::instrument(skip(self))]
    pub fn purge_deleted(&self) -> anyhow::Result<()> {
        let pending = self.load_pending()?;
        let before_count = pending.len();
        let kept: Vec<Task> = pending
            .into_iter()
            .filter(|task| task.status != Status::Deleted)
            .collect();
        info!(
            before = before_count,
            after = kept.len(),
            "purged deleted tasks"
        );
        self.save_pending(&kept)
    }
}

#[tracing::instrument(skip(path))]
fn load_jsonl(path: &Path) -> anyhow::Result<Vec<Task>> {
    debug!(file = %path.display(), "loading jsonl");
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut out = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let task: Task = serde_json::from_str(trimmed)
            .with_context(|| format!("failed parsing {} line {}", path.display(), idx + 1))?;
        out.push(task);
    }

    debug!(count = out.len(), "loaded tasks from jsonl");
    Ok(out)
}

#[tracing::instrument(skip(path, tasks))]
fn save_jsonl_atomic(path: &Path, tasks: &[Task]) -> anyhow::Result<()> {
    debug!(file = %path.display(), count = tasks.len(), "saving jsonl atomically");

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp = NamedTempFile::new_in(dir)?;
    for task in tasks {
        let serialized = serde_json::to_string(task)?;
        writeln!(temp, "{serialized}")?;
    }
    temp.flush()?;

    temp.persist(path)
        .map_err(|err| anyhow!("failed to persist {}: {}", path.display(), err))?;

    Ok(())
}
