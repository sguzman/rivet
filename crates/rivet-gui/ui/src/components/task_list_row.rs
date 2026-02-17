use std::collections::{
  BTreeMap,
  BTreeSet
};

use rivet_gui_shared::{
  TaskDto,
  TaskStatus
};
use uuid::Uuid;
use yew::{
  Callback,
  Html,
  Properties,
  function_component,
  html
};

use super::TaskTagBadge;

#[derive(Properties, PartialEq)]
pub struct TaskListRowProps {
  pub task:             TaskDto,
  pub tag_colors:
    BTreeMap<String, String>,
  pub selected:         Option<Uuid>,
  pub selected_ids:     BTreeSet<Uuid>,
  pub on_select:        Callback<Uuid>,
  pub on_toggle_select: Callback<Uuid>
}

#[function_component(TaskListRow)]
pub fn task_list_row(
  props: &TaskListRowProps
) -> Html {
  let id = props.task.uuid;
  let selected =
    props.selected == Some(id);
  let class = if selected {
    "row selected"
  } else {
    "row"
  };
  let on_select =
    props.on_select.clone();
  let on_toggle_select =
    props.on_toggle_select.clone();
  let checked =
    props.selected_ids.contains(&id);

  let dot_class =
    match props.task.status {
      | TaskStatus::Pending => {
        "dot pending"
      }
      | TaskStatus::Completed => {
        "dot done"
      }
      | TaskStatus::Deleted => {
        "dot deleted"
      }
      | TaskStatus::Waiting => {
        "dot waiting"
      }
    };

  let meta_project = props
    .task
    .project
    .clone()
    .unwrap_or_else(|| "—".to_string());
  let due = props
    .task
    .due
    .clone()
    .unwrap_or_default();
  let has_description = !props
    .task
    .description
    .trim()
    .is_empty();

  html! {
      <div class={class} onclick={move |_| on_select.emit(id)}>
          <button
              class={if checked { "selector on" } else { "selector" }}
              onclick={move |e: yew::MouseEvent| {
                  e.stop_propagation();
                  on_toggle_select.emit(id);
              }}
          >
              { if checked { "✓" } else { "" } }
          </button>
          <div class={dot_class}></div>
          <div>
              <div>{ &props.task.title }</div>
              {
                  if has_description {
                      html! { <div class="task-subtitle">{ &props.task.description }</div> }
                  } else {
                      html! {}
                  }
              }
              <div style="margin-top:4px;display:flex;gap:6px;flex-wrap:wrap;">
                  <span class="badge">{ format!("project:{meta_project}") }</span>
                  {
                      for props.task.tags.iter().take(4).cloned().map(|tag| html! {
                          <TaskTagBadge tag={tag} tag_colors={props.tag_colors.clone()} />
                      })
                  }
              </div>
          </div>
          <div>
              {
                  if due.is_empty() {
                      html! {}
                  } else {
                      html! { <span class="badge">{ format!("due:{due}") }</span> }
                  }
              }
          </div>
      </div>
  }
}
