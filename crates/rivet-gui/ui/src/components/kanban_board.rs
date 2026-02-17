use std::collections::BTreeMap;

use rivet_gui_shared::TaskDto;
use uuid::Uuid;
use yew::{
  Callback,
  Html,
  Properties,
  function_component,
  html
};

use super::KanbanColumn;

#[derive(Properties, PartialEq)]
pub struct KanbanBoardProps {
  pub tasks:             Vec<TaskDto>,
  pub columns:           Vec<String>,
  pub board_name:        Option<String>,
  pub tag_colors:
    BTreeMap<String, String>,
  pub compact_cards:     bool,
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

#[function_component(KanbanBoard)]
pub fn kanban_board(
  props: &KanbanBoardProps
) -> Html {
  let columns =
    if props.columns.is_empty() {
      vec![
        "todo".to_string(),
        "working".to_string(),
        "finished".to_string(),
      ]
    } else {
      props.columns.clone()
    };
  let default_lane = columns
    .first()
    .cloned()
    .unwrap_or_else(|| {
      "todo".to_string()
    });

  let board_label = props
    .board_name
    .clone()
    .unwrap_or_else(|| {
      "No board selected".to_string()
    });

  html! {
      <div class="panel kanban-panel">
          <div class="header">{ format!("Kanban: {board_label}") }</div>
          <div class="kanban-board">
              {
                  for columns.iter().enumerate().map(|(column_idx, column_key)| {
                      let cards: Vec<TaskDto> = props
                          .tasks
                          .iter()
                          .filter(|task| {
                              kanban_lane_for_task(
                                  task,
                                  &columns,
                                  &default_lane,
                              ) == *column_key
                          })
                          .cloned()
                          .collect();

                      html! {
                          <KanbanColumn
                              column_key={column_key.clone()}
                              column_idx={column_idx}
                              columns={columns.clone()}
                              cards={cards}
                              compact_cards={props.compact_cards}
                              tag_colors={props.tag_colors.clone()}
                              dragging_task={props.dragging_task}
                              drag_over_lane={props.drag_over_lane.clone()}
                              on_move={props.on_move.clone()}
                              on_drag_start={props.on_drag_start.clone()}
                              on_drag_end={props.on_drag_end.clone()}
                              on_drag_over_lane={props.on_drag_over_lane.clone()}
                              on_edit={props.on_edit.clone()}
                              on_done={props.on_done.clone()}
                              on_delete={props.on_delete.clone()}
                          />
                      }
                  })
              }
          </div>
      </div>
  }
}

fn kanban_lane_for_task(
  task: &TaskDto,
  columns: &[String],
  default_lane: &str
) -> String {
  for tag in &task.tags {
    if let Some((key, value)) =
      tag.split_once(':')
      && key == "kanban"
    {
      if columns
        .iter()
        .any(|column| column == value)
      {
        return value.to_string();
      }
      return default_lane.to_string();
    }
  }
  default_lane.to_string()
}

pub(crate) fn humanize_lane(
  value: &str
) -> String {
  value
    .split(['-', '_'])
    .filter(|part| !part.is_empty())
    .map(|part| {
      let mut chars = part.chars();
      match chars.next() {
        | Some(first) => {
          let mut out = first
            .to_ascii_uppercase()
            .to_string();
          out.push_str(
            &chars
              .as_str()
              .to_ascii_lowercase()
          );
          out
        }
        | None => String::new()
      }
    })
    .collect::<Vec<_>>()
    .join(" ")
}
