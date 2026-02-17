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
  KanbanCard,
  humanize_lane
};

#[derive(Properties, PartialEq)]
pub struct KanbanColumnProps {
  pub column_key:        String,
  pub column_idx:        usize,
  pub columns:           Vec<String>,
  pub cards:             Vec<TaskDto>,
  pub compact_cards:     bool,
  pub tag_colors:
    BTreeMap<String, String>,
  pub dragging_task:     Option<Uuid>,
  pub drag_over_lane:    Option<String>,
  pub on_move: Callback<(Uuid, String)>,
  pub on_drag_start:     Callback<Uuid>,
  pub on_drag_end:       Callback<()>,
  pub on_drag_over_lane:
    Callback<String>,
  pub on_edit: Callback<TaskDto>,
  pub on_done:           Callback<Uuid>,
  pub on_delete:         Callback<Uuid>
}

#[function_component(KanbanColumn)]
pub fn kanban_column(
  props: &KanbanColumnProps
) -> Html {
  let column_title =
    humanize_lane(&props.column_key);
  let lane_for_dragover =
    props.column_key.clone();
  let lane_for_dragenter =
    props.column_key.clone();
  let column_key_string =
    props.column_key.clone();
  let is_drop_hint = props
    .drag_over_lane
    .as_deref()
    == Some(props.column_key.as_str());

  let ondragover = {
    let on_drag_over_lane =
      props.on_drag_over_lane.clone();
    Callback::from(
      move |event: DragEvent| {
        event.prevent_default();
        event.stop_propagation();
        on_drag_over_lane.emit(
          lane_for_dragover.clone()
        );
      }
    )
  };

  let ondragenter = {
    let on_drag_over_lane =
      props.on_drag_over_lane.clone();
    Callback::from(
      move |event: DragEvent| {
        event.prevent_default();
        event.stop_propagation();
        on_drag_over_lane.emit(
          lane_for_dragenter.clone()
        );
      }
    )
  };

  let ondrop = {
    let on_move = props.on_move.clone();
    let on_drag_end =
      props.on_drag_end.clone();
    Callback::from(
      move |event: DragEvent| {
        event.prevent_default();
        event.stop_propagation();
        if let Some(data_transfer) =
          event.data_transfer()
        {
          match data_transfer
            .get_data("text/plain")
          {
            | Ok(raw_uuid) => {
              if let Ok(uuid) =
                Uuid::parse_str(
                  raw_uuid.trim()
                )
              {
                on_move.emit((
                  uuid,
                  column_key_string
                    .clone()
                ));
              } else {
                tracing::warn!(
                  raw_uuid,
                  "failed to parse \
                   dragged task uuid"
                );
              }
            }
            | Err(error) => {
              tracing::warn!(
                ?error,
                "failed reading drag \
                 data"
              )
            }
          }
        }
        on_drag_end.emit(());
      }
    )
  };

  html! {
      <div class={classes!("kanban-column", is_drop_hint.then_some("drop-hint"))} {ondragover} {ondragenter} {ondrop}>
          <div class="kanban-column-header">
              <span>{ column_title }</span>
              <span class="badge">{ props.cards.len() }</span>
          </div>
          <div class="kanban-column-body">
              {
                  if props.cards.is_empty() {
                      html! { <div class="kanban-empty">{ "No tasks" }</div> }
                  } else {
                      html! {
                          <>
                              {
                                  for props.cards.iter().cloned().map(|task| {
                                      let task_id = task.uuid;
                                      let next_lane = if props.columns.is_empty() {
                                          props.column_key.clone()
                                      } else {
                                          props.columns[(props.column_idx + 1) % props.columns.len()].clone()
                                      };
                                      let next_lane_label = format!("Move to {}", humanize_lane(&next_lane));
                                      html! {
                                          <KanbanCard
                                              task={task}
                                              is_dragging={props.dragging_task == Some(task_id)}
                                              compact_cards={props.compact_cards}
                                              next_lane={next_lane}
                                              next_lane_label={next_lane_label}
                                              tag_colors={props.tag_colors.clone()}
                                              on_drag_start={props.on_drag_start.clone()}
                                              on_drag_end={props.on_drag_end.clone()}
                                              on_edit={props.on_edit.clone()}
                                              on_move={props.on_move.clone()}
                                              on_done={props.on_done.clone()}
                                              on_delete={props.on_delete.clone()}
                                          />
                                      }
                                  })
                              }
                          </>
                      }
                  }
              }
          </div>
      </div>
  }
}
