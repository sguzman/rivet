use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, anyhow};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tempfile::TempDir;
use tracing::{debug, info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "rivet-parity",
    about = "Runs parity scenarios against Rivet task CLI and Taskwarrior"
)]
struct Args {
    #[arg(long, default_value = "target/debug/task")]
    candidate_bin: PathBuf,

    #[arg(long, default_value = "task")]
    reference_bin: PathBuf,

    #[arg(long, default_value = "crates/rivet-parity/scenarios/basic_flow.json")]
    scenario: Vec<PathBuf>,

    #[arg(long)]
    skip_reference: bool,

    #[arg(long, default_value = "warn")]
    log_level: String,
}

#[derive(Debug, Deserialize)]
struct Scenario {
    name: String,
    steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
struct Step {
    args: Vec<String>,
    #[serde(default)]
    stdin: Option<String>,
}

#[derive(Debug)]
struct EngineResult {
    pending: Vec<CanonicalTask>,
    completed: Vec<CanonicalTask>,
    deleted: Vec<CanonicalTask>,
}

#[derive(Debug)]
struct StepResult {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
struct CanonicalTask {
    description: String,
    status: String,
    project: Option<String>,
    tags: Vec<String>,
    priority: Option<String>,
    due: Option<String>,
    scheduled: Option<String>,
    wait: Option<String>,
    start: Option<String>,
    annotations: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_tracing(&args.log_level)?;

    let scenarios = load_scenarios(&args.scenario)?;
    if scenarios.is_empty() {
        return Err(anyhow!("no scenarios loaded"));
    }

    let reference_available = !args.skip_reference && is_reference_available(&args.reference_bin);
    if !reference_available {
        warn!(
            reference = %args.reference_bin.display(),
            "reference binary unavailable or skipped; running candidate only"
        );
    }

    let mut overall_scores = Vec::new();

    for scenario in scenarios {
        info!(scenario = %scenario.name, "running scenario");

        let candidate = run_engine(&args.candidate_bin, &scenario)
            .with_context(|| format!("candidate failed on scenario {}", scenario.name))?;

        if reference_available {
            let reference = run_engine(&args.reference_bin, &scenario)
                .with_context(|| format!("reference failed on scenario {}", scenario.name))?;

            let pending_score = score_bucket(&candidate.pending, &reference.pending);
            let completed_score = score_bucket(&candidate.completed, &reference.completed);
            let deleted_score = score_bucket(&candidate.deleted, &reference.deleted);
            let scenario_score = (pending_score + completed_score + deleted_score) / 3.0;

            overall_scores.push(scenario_score);

            println!("Scenario: {}", scenario.name);
            println!("  pending parity  : {:.3}", pending_score);
            println!("  completed parity: {:.3}", completed_score);
            println!("  deleted parity  : {:.3}", deleted_score);
            println!("  scenario parity : {:.3}", scenario_score);

            print_diff("pending", &candidate.pending, &reference.pending);
            print_diff("completed", &candidate.completed, &reference.completed);
            print_diff("deleted", &candidate.deleted, &reference.deleted);
        } else {
            println!("Scenario: {}", scenario.name);
            println!("  reference skipped; candidate produced:");
            println!("    pending:   {}", candidate.pending.len());
            println!("    completed: {}", candidate.completed.len());
            println!("    deleted:   {}", candidate.deleted.len());
        }
    }

    if !overall_scores.is_empty() {
        let mean = overall_scores.iter().sum::<f64>() / overall_scores.len() as f64;
        println!("\nOverall parity score: {:.3}", mean);
    }

    Ok(())
}

fn init_tracing(level: &str) -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_new(level)
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("warn"))
        .map_err(|e| anyhow!("invalid log level: {e}"))?;

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_ansi(std::io::stderr().is_terminal())
        .try_init();

    Ok(())
}

fn load_scenarios(paths: &[PathBuf]) -> anyhow::Result<Vec<Scenario>> {
    let mut out = Vec::new();

    for path in paths {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read scenario {}", path.display()))?;
        let scenario: Scenario = serde_json::from_str(&text)
            .with_context(|| format!("failed to parse scenario {}", path.display()))?;
        out.push(scenario);
    }

    Ok(out)
}

fn is_reference_available(reference_bin: &Path) -> bool {
    if reference_bin.is_absolute() || reference_bin.components().count() > 1 {
        return reference_bin.exists();
    }

    Command::new(reference_bin)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn run_engine(binary: &Path, scenario: &Scenario) -> anyhow::Result<EngineResult> {
    let temp_dir = TempDir::new().context("failed to create temp scenario dir")?;
    let taskrc = write_taskrc(temp_dir.path())?;

    for (idx, step) in scenario.steps.iter().enumerate() {
        let result = run_command(binary, &taskrc, &step.args, step.stdin.as_deref())?;
        if !result.status.success() {
            return Err(anyhow!(
                "command failed in scenario {} step {}: {:?}\nstdout: {}\nstderr: {}",
                scenario.name,
                idx + 1,
                step.args,
                result.stdout,
                result.stderr
            ));
        }
    }

    let mut pending = collect_export(binary, &taskrc, &["status:pending"])?;
    pending.extend(collect_export(binary, &taskrc, &["status:waiting"])?);
    pending.sort();
    pending.dedup();
    let completed = collect_export(binary, &taskrc, &["status:completed"])?;
    let deleted = collect_export(binary, &taskrc, &["status:deleted"])?;

    Ok(EngineResult {
        pending,
        completed,
        deleted,
    })
}

fn write_taskrc(base: &Path) -> anyhow::Result<PathBuf> {
    let data_dir = base.join("data");
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("failed to create data dir {}", data_dir.display()))?;

    let taskrc = base.join("taskrc");
    let content = format!(
        "data.location={}\nconfirmation=no\nverbose=nothing\ncontext.rivet=+rivet\ncontext.ops=project:ops\nreport.focus.columns=id,description,urgency\nreport.focus.labels=ID,Description,Urgency\nreport.focus.sort=urgency-\nreport.focus.filter=status:pending\nreport.focus.limit=50\n",
        data_dir.display()
    );
    fs::write(&taskrc, content)
        .with_context(|| format!("failed to write taskrc {}", taskrc.display()))?;

    Ok(taskrc)
}

fn run_command(
    binary: &Path,
    taskrc: &Path,
    args: &[String],
    stdin: Option<&str>,
) -> anyhow::Result<StepResult> {
    let mut cmd = Command::new(binary);
    cmd.args(args)
        .env("TASKRC", taskrc)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if stdin.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }

    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to start {} {:?}", binary.display(), args))?;

    if let Some(input) = stdin {
        use std::io::Write;
        if let Some(mut pipe) = child.stdin.take() {
            pipe.write_all(input.as_bytes())
                .context("failed to write command stdin")?;
        }
    }

    let output = child
        .wait_with_output()
        .context("failed waiting for command")?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    debug!(
        binary = %binary.display(),
        args = ?args,
        success = output.status.success(),
        stdout = %stdout,
        stderr = %stderr,
        "command completed"
    );

    Ok(StepResult {
        status: output.status,
        stdout,
        stderr,
    })
}

fn collect_export(
    binary: &Path,
    taskrc: &Path,
    filter_terms: &[&str],
) -> anyhow::Result<Vec<CanonicalTask>> {
    let mut cmd = Command::new(binary);
    for term in filter_terms {
        cmd.arg(term);
    }
    cmd.arg("export").env("TASKRC", taskrc);

    let output = cmd
        .output()
        .with_context(|| format!("failed to run export for {}", binary.display()))?;

    if !output.status.success() {
        return Err(anyhow!(
            "export failed for {}: {}",
            binary.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw.is_empty() {
        return Ok(vec![]);
    }

    let parsed: Vec<Value> = serde_json::from_str(&raw)
        .with_context(|| format!("invalid export json from {}: {}", binary.display(), raw))?;

    let mut tasks = parsed
        .into_iter()
        .map(canonicalize)
        .collect::<anyhow::Result<Vec<_>>>()?;
    tasks.sort();
    Ok(tasks)
}

fn canonicalize(value: Value) -> anyhow::Result<CanonicalTask> {
    let obj = value
        .as_object()
        .ok_or_else(|| anyhow!("expected task object in export"))?;

    let description = obj
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let status = obj
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("pending")
        .to_string();

    let project = obj
        .get("project")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let mut tags = obj
        .get("tags")
        .and_then(Value::as_array)
        .map(|tags| {
            tags.iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    tags.sort();

    let priority = obj
        .get("priority")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let due = obj
        .get("due")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let scheduled = obj
        .get("scheduled")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let wait = obj
        .get("wait")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let start = obj
        .get("start")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let mut annotations = obj
        .get("annotations")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("description").and_then(Value::as_str))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    annotations.sort();

    Ok(CanonicalTask {
        description,
        status,
        project,
        tags,
        priority,
        due,
        scheduled,
        wait,
        start,
        annotations,
    })
}

fn score_bucket(candidate: &[CanonicalTask], reference: &[CanonicalTask]) -> f64 {
    let c: BTreeSet<_> = candidate.iter().cloned().collect();
    let r: BTreeSet<_> = reference.iter().cloned().collect();

    if c.is_empty() && r.is_empty() {
        return 1.0;
    }

    let intersection = c.intersection(&r).count() as f64;
    let union = c.union(&r).count() as f64;

    if union == 0.0 {
        1.0
    } else {
        intersection / union
    }
}

fn print_diff(label: &str, candidate: &[CanonicalTask], reference: &[CanonicalTask]) {
    let c: BTreeSet<_> = candidate.iter().cloned().collect();
    let r: BTreeSet<_> = reference.iter().cloned().collect();

    let only_candidate: Vec<_> = c.difference(&r).cloned().collect();
    let only_reference: Vec<_> = r.difference(&c).cloned().collect();

    if only_candidate.is_empty() && only_reference.is_empty() {
        return;
    }

    println!("  {label} diff:");
    if !only_candidate.is_empty() {
        println!("    only candidate:");
        for task in only_candidate {
            println!("      {} [{:?}]", task.description, task.tags);
        }
    }
    if !only_reference.is_empty() {
        println!("    only reference:");
        for task in only_reference {
            println!("      {} [{:?}]", task.description, task.tags);
        }
    }
}

use std::io::IsTerminal;

#[cfg(test)]
mod tests {
    use super::{CanonicalTask, canonicalize, score_bucket};
    use serde_json::json;

    #[test]
    fn canonicalize_extracts_expected_fields() {
        let value = json!({
            "description": "Demo",
            "status": "pending",
            "project": "rivet",
            "tags": ["gui", "core"],
            "priority": "H",
            "due": "20260217T010000Z"
        });

        let task = canonicalize(value).expect("canonicalize should succeed");
        assert_eq!(task.description, "Demo");
        assert_eq!(task.status, "pending");
        assert_eq!(task.project.as_deref(), Some("rivet"));
        assert_eq!(task.tags, vec!["core".to_string(), "gui".to_string()]);
        assert_eq!(task.priority.as_deref(), Some("H"));
    }

    #[test]
    fn score_bucket_uses_jaccard_similarity() {
        let a = vec![CanonicalTask {
            description: "A".to_string(),
            status: "pending".to_string(),
            project: None,
            tags: vec![],
            priority: None,
            due: None,
            scheduled: None,
            wait: None,
            start: None,
            annotations: vec![],
        }];
        let b = vec![CanonicalTask {
            description: "A".to_string(),
            status: "pending".to_string(),
            project: None,
            tags: vec![],
            priority: None,
            due: None,
            scheduled: None,
            wait: None,
            start: None,
            annotations: vec![],
        }];

        assert!((score_bucket(&a, &b) - 1.0).abs() < f64::EPSILON);
    }
}
