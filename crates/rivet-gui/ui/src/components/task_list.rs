use std::collections::{
  BTreeMap,
  BTreeSet
};

use rivet_gui_shared::TaskDto;
use uuid::Uuid;
use yew::{
  Callback,
  Html,
  Properties,
  function_component,
  html
};

use super::TaskListRow;

#[derive(Properties, PartialEq)]
pub struct TaskListProps {
  pub tasks:            Vec<TaskDto>,
  pub tag_colors:
    BTreeMap<String, String>,
  pub selected:         Option<Uuid>,
  pub selected_ids:     BTreeSet<Uuid>,
  pub on_select:        Callback<Uuid>,
  pub on_toggle_select: Callback<Uuid>
}

#[function_component(TaskList)]
pub fn task_list(
  props: &TaskListProps
) -> Html {
  html! {
      <div class="panel list">
          <div class="header">{ "Tasks" }</div>
          {
              for props.tasks.iter().cloned().map(|task| html! {
                  <TaskListRow
                      task={task}
                      tag_colors={props.tag_colors.clone()}
                      selected={props.selected}
                      selected_ids={props.selected_ids.clone()}
                      on_select={props.on_select.clone()}
                      on_toggle_select={props.on_toggle_select.clone()}
                  />
              })
          }
      </div>
  }
}
