use std::collections::BTreeMap;

use rivet_gui_shared::TaskDto;
use uuid::Uuid;
use web_sys::DragEvent;
use yew::{
  Callback,
  Html,
  Properties,
  classes,
  function_component,
  html
};

use super::{
  KanbanCardActions,
  KanbanCardMeta
};

#[derive(Properties, PartialEq)]
pub struct KanbanCardProps {
  pub task:            TaskDto,
  pub is_dragging:     bool,
  pub compact_cards:   bool,
  pub next_lane:       String,
  pub next_lane_label: String,
  pub tag_colors:
    BTreeMap<String, String>,
  pub on_drag_start:   Callback<Uuid>,
  pub on_drag_end:     Callback<()>,
  pub on_edit: Callback<TaskDto>,
  pub on_move: Callback<(Uuid, String)>,
  pub on_done:         Callback<Uuid>,
  pub on_delete:       Callback<Uuid>
}

#[function_component(KanbanCard)]
pub fn kanban_card(
  props: &KanbanCardProps
) -> Html {
  let task_id = props.task.uuid;
  let ondragstart = {
    let on_drag_start =
      props.on_drag_start.clone();
    Callback::from(
      move |event: DragEvent| {
        if let Some(data_transfer) =
          event.data_transfer()
        {
          let _ = data_transfer
            .set_data(
              "text/plain",
              &task_id.to_string()
            );
          data_transfer
            .set_drop_effect("move");
        }
        on_drag_start.emit(task_id);
      }
    )
  };

  let ondragend = {
    let on_drag_end =
      props.on_drag_end.clone();
    Callback::from(move |_| {
      on_drag_end.emit(());
    })
  };

  html! {
      <div class={classes!("kanban-card", props.is_dragging.then_some("dragging"))} draggable="true" {ondragstart} {ondragend}>
          <div class="kanban-card-title">{ &props.task.title }</div>
          {
              if props.task.description.trim().is_empty() {
                  html! {}
              } else {
                  html! { <div class="task-subtitle">{ &props.task.description }</div> }
              }
          }
          {
              if props.compact_cards {
                  html! {}
              } else {
                  html! {
                      <>
                          <KanbanCardMeta
                              task={props.task.clone()}
                              tag_colors={props.tag_colors.clone()}
                          />
                          <KanbanCardActions
                              task_id={task_id}
                              task_status={props.task.status.clone()}
                              task_for_edit={props.task.clone()}
                              next_lane={props.next_lane.clone()}
                              next_lane_label={props.next_lane_label.clone()}
                              on_edit={props.on_edit.clone()}
                              on_move={props.on_move.clone()}
                              on_done={props.on_done.clone()}
                              on_delete={props.on_delete.clone()}
                          />
                      </>
                  }
              }
          }
      </div>
  }
}
