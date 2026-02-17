use std::collections::BTreeMap;

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
pub struct DetailsProps {
  pub task:       Option<TaskDto>,
  pub tag_colors:
    BTreeMap<String, String>,
  pub on_done:    Callback<Uuid>,
  pub on_delete:  Callback<Uuid>,
  pub on_edit:    Callback<TaskDto>
}

#[function_component(Details)]
pub fn details(
  props: &DetailsProps
) -> Html {
  let Some(task) = &props.task else {
    return html! {
        <div class="panel">
            <div class="header">{ "Details" }</div>
            <div class="details">
                <div style="color:var(--muted);">{ "Select a task to inspect and edit details." }</div>
            </div>
        </div>
    };
  };

  let on_done = props.on_done.clone();
  let on_delete =
    props.on_delete.clone();
  let on_edit = props.on_edit.clone();
  let id = task.uuid;
  let task_for_edit = task.clone();
  let can_mark_done = matches!(
    task.status,
    TaskStatus::Pending
      | TaskStatus::Waiting
  );

  html! {
      <div class="panel">
          <div class="header">{ "Details" }</div>
          <div class="details">
              <div style="font-family:var(--mono);color:var(--muted);">{ format!("uuid: {id}") }</div>
              <div style="font-size:1.15rem;font-weight:700;">{ &task.title }</div>
              {
                  if task.description.trim().is_empty() {
                      html! {}
                  } else {
                      html! { <div class="task-subtitle">{ &task.description }</div> }
                  }
              }

              <div class="kv">
                  <strong>{ "project" }</strong>
                  <div>{ task.project.clone().unwrap_or_else(|| "—".to_string()) }</div>
              </div>

              <div class="kv">
                  <strong>{ "tags" }</strong>
                  <div>
                      {
                          if task.tags.is_empty() {
                              html! { <span style="color:var(--muted);">{ "—" }</span> }
                          } else {
                              html! {
                                  <>
                                      {
                                          for task.tags.iter().cloned().map(|tag| html!{
                                              <span style="margin-right:6px;">
                                                  <TaskTagBadge tag={tag} tag_colors={props.tag_colors.clone()} />
                                              </span>
                                          })
                                      }
                                  </>
                              }
                          }
                      }
                  </div>
              </div>

              <div class="kv">
                  <strong>{ "due" }</strong>
                  <div>{ task.due.clone().unwrap_or_else(|| "—".to_string()) }</div>
              </div>

              <div class="kv">
                  <strong>{ "status" }</strong>
                  <div>{ format!("{:?}", task.status) }</div>
              </div>

              <div class="actions">
                  <button class="btn" onclick={move |_| on_edit.emit(task_for_edit.clone())}>{ "Edit" }</button>
                  {
                      if can_mark_done {
                          html! { <button class="btn ok" onclick={move |_| on_done.emit(id)}>{ "Done" }</button> }
                      } else {
                          html! {}
                      }
                  }
                  <button class="btn danger" onclick={move |_| on_delete.emit(id)}>{ "Delete" }</button>
              </div>
          </div>
      </div>
  }
}
