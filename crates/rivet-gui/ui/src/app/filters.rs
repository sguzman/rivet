fn filter_visible_tasks(
  tasks: &[TaskDto],
  active_view: &str,
  query: &str,
  active_project: Option<&str>,
  active_tag: Option<&str>,
  all_filter_completion: &str,
  all_filter_project: Option<&str>,
  all_filter_tag: Option<&str>,
  all_filter_priority: &str,
  all_filter_due: &str
) -> Vec<TaskDto> {
  let q = query.to_ascii_lowercase();

  tasks
    .iter()
    .filter(|task| {
      if !q.is_empty() {
        let title_match = task
          .title
          .to_ascii_lowercase()
          .contains(&q);
        let description_match = task
          .description
          .to_ascii_lowercase()
          .contains(&q);
        if !title_match
          && !description_match
        {
          return false;
        }
      }

      match active_view {
        | "projects" => {
          if let Some(project) =
            active_project
          {
            task.project.as_deref()
              == Some(project)
          } else {
            true
          }
        }
        | "tags" => {
          if let Some(tag) = active_tag
          {
            task
              .tags
              .iter()
              .any(|value| value == tag)
          } else {
            true
          }
        }
        | "all" | "kanban" => {
          if let Some(project) =
            all_filter_project
            && task.project.as_deref()
              != Some(project)
          {
            return false;
          }

          if let Some(tag) = all_filter_tag
            && !task
              .tags
              .iter()
              .any(|value| value == tag)
          {
            return false;
          }

          let completion_match =
            match all_filter_completion {
            | "open" => matches!(
              task.status,
              TaskStatus::Pending
                | TaskStatus::Waiting
            ),
            | "pending" => {
              task.status
                == TaskStatus::Pending
            }
            | "waiting" => {
              task.status
                == TaskStatus::Waiting
            }
            | "completed" => {
              task.status
                == TaskStatus::Completed
            }
            | "deleted" => {
              task.status
                == TaskStatus::Deleted
            }
            | _ => true
          };

          let priority_match =
            match all_filter_priority {
            | "low" => task.priority
              == Some(
                rivet_gui_shared::TaskPriority::Low
              ),
            | "medium" => task.priority
              == Some(
                rivet_gui_shared::TaskPriority::Medium
              ),
            | "high" => task.priority
              == Some(
                rivet_gui_shared::TaskPriority::High
              ),
            | "none" => {
              task.priority.is_none()
            }
            | _ => true
          };

          let due_match = match all_filter_due {
            | "has_due" => {
              task.due.is_some()
            }
            | "no_due" => task.due.is_none(),
            | _ => true
          };

          completion_match
            && priority_match
            && due_match
        }
        | _ => true
      }
    })
    .cloned()
    .collect()
}

fn build_project_facets(
  tasks: &[TaskDto]
) -> Vec<(String, usize)> {
  let mut counts = BTreeMap::new();
  for task in tasks {
    if let Some(project) =
      task.project.as_ref()
    {
      *counts
        .entry(project.clone())
        .or_insert(0_usize) += 1;
    }
  }
  counts.into_iter().collect()
}

fn build_tag_facets(
  tasks: &[TaskDto]
) -> Vec<(String, usize)> {
  let mut counts = BTreeMap::new();
  for task in tasks {
    for tag in &task.tags {
      *counts
        .entry(tag.clone())
        .or_insert(0_usize) += 1;
    }
  }
  counts.into_iter().collect()
}

fn ui_debug(
  event: &str,
  detail: &str
) {
  tracing::debug!(
    event, detail, "ui-debug"
  );
  log!(format!(
    "[ui-debug] {event}: {detail}"
  ));
}
