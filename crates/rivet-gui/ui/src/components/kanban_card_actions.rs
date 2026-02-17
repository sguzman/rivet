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

#[derive(Properties, PartialEq)]
pub struct KanbanCardActionsProps {
  pub task_id:         Uuid,
  pub task_status:     TaskStatus,
  pub task_for_edit:   TaskDto,
  pub next_lane:       String,
  pub next_lane_label: String,
  pub on_edit: Callback<TaskDto>,
  pub on_move: Callback<(Uuid, String)>,
  pub on_done:         Callback<Uuid>,
  pub on_delete:       Callback<Uuid>
}

#[function_component(KanbanCardActions)]
pub fn kanban_card_actions(
  props: &KanbanCardActionsProps
) -> Html {
  let task_id = props.task_id;
  let next_lane =
    props.next_lane.clone();
  html! {
      <div class="kanban-card-actions">
          <button class="btn" onclick={{
              let on_edit = props.on_edit.clone();
              let task_for_edit = props.task_for_edit.clone();
              Callback::from(move |_| on_edit.emit(task_for_edit.clone()))
          }}>{ "Edit" }</button>
          <button class="btn" onclick={{
              let on_move = props.on_move.clone();
              let next_lane = next_lane.clone();
              Callback::from(move |_| on_move.emit((task_id, next_lane.clone())))
          }}>{ props.next_lane_label.clone() }</button>
          {
              if matches!(props.task_status, TaskStatus::Pending | TaskStatus::Waiting) {
                  html! { <button class="btn ok" onclick={{
                      let on_done = props.on_done.clone();
                      Callback::from(move |_| on_done.emit(task_id))
                  }}>{ "Done" }</button> }
              } else {
                  html! {}
              }
          }
          <button class="btn danger" onclick={{
              let on_delete = props.on_delete.clone();
              Callback::from(move |_| on_delete.emit(task_id))
          }}>{ "Delete" }</button>
      </div>
  }
}
