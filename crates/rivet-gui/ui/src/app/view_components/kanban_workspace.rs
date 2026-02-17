#[derive(Properties, PartialEq)]
struct KanbanWorkspaceProps {
  kanban_boards:           Vec<KanbanBoardDef>,
  active_kanban_board_id:  Option<String>,
  active_kanban_board_name:
    Option<String>,
  kanban_compact_cards:    bool,
  on_create_kanban_board:  Callback<MouseEvent>,
  on_open_rename_kanban_board:
    Callback<MouseEvent>,
  on_delete_kanban_board:  Callback<MouseEvent>,
  on_toggle_card_density:  Callback<MouseEvent>,
  on_select_kanban_board:  Callback<String>,
  kanban_visible_tasks:    Vec<TaskDto>,
  kanban_columns:          Vec<String>,
  tag_colors:
    BTreeMap<String, String>,
  dragging_task:           Option<Uuid>,
  drag_over_lane:          Option<String>,
  on_kanban_move:
    Callback<(Uuid, String)>,
  on_kanban_drag_start:
    Callback<Uuid>,
  on_kanban_drag_end:
    Callback<()>,
  on_kanban_drag_over_lane:
    Callback<String>,
  on_edit:                 Callback<TaskDto>,
  on_done:                 Callback<Uuid>,
  on_delete:               Callback<Uuid>,
  completion_value:        String,
  on_completion_change:
    Callback<web_sys::Event>,
  project_value:           String,
  project_items:           Vec<(String, usize)>,
  on_project_change:
    Callback<web_sys::Event>,
  tag_value:               String,
  tag_items:               Vec<(String, usize)>,
  on_tag_change:
    Callback<web_sys::Event>,
  priority_value:          String,
  on_priority_change:
    Callback<web_sys::Event>,
  due_value:               String,
  on_due_change:
    Callback<web_sys::Event>,
  on_clear_filters:
    Callback<MouseEvent>
}

#[function_component(KanbanWorkspace)]
fn kanban_workspace(
  props: &KanbanWorkspaceProps
) -> Html {
  html! {
      <>
          <KanbanBoardsSidebar
              boards={props.kanban_boards.clone()}
              active_board_id={props.active_kanban_board_id.clone()}
              compact_cards={props.kanban_compact_cards}
              on_create_kanban_board={props.on_create_kanban_board.clone()}
              on_open_rename_kanban_board={props.on_open_rename_kanban_board.clone()}
              on_delete_kanban_board={props.on_delete_kanban_board.clone()}
              on_toggle_card_density={props.on_toggle_card_density.clone()}
              on_select_board_id={props.on_select_kanban_board.clone()}
          />

          <KanbanBoard
              tasks={props.kanban_visible_tasks.clone()}
              columns={props.kanban_columns.clone()}
              board_name={props.active_kanban_board_name.clone()}
              tag_colors={props.tag_colors.clone()}
              compact_cards={props.kanban_compact_cards}
              dragging_task={props.dragging_task}
              drag_over_lane={props.drag_over_lane.clone()}
              on_move={props.on_kanban_move.clone()}
              on_drag_start={props.on_kanban_drag_start.clone()}
              on_drag_end={props.on_kanban_drag_end.clone()}
              on_drag_over_lane={props.on_kanban_drag_over_lane.clone()}
              on_edit={props.on_edit.clone()}
              on_done={props.on_done.clone()}
              on_delete={props.on_delete.clone()}
          />

          <div class="right-stack">
              <div class="panel">
                  <div class="header">{ "Kanban Summary" }</div>
                  <div class="details">
                      <div class="kv">
                          <strong>{ "board" }</strong>
                          <div>{ props.active_kanban_board_name.clone().unwrap_or_else(|| "None".to_string()) }</div>
                      </div>
                      <div class="kv">
                          <strong>{ "cards shown" }</strong>
                          <div>{ props.kanban_visible_tasks.len() }</div>
                      </div>
                  </div>
              </div>
              <KanbanFiltersPanel
                  completion_value={props.completion_value.clone()}
                  on_completion_change={props.on_completion_change.clone()}
                  project_value={props.project_value.clone()}
                  project_items={props.project_items.clone()}
                  on_project_change={props.on_project_change.clone()}
                  tag_value={props.tag_value.clone()}
                  tag_items={props.tag_items.clone()}
                  on_tag_change={props.on_tag_change.clone()}
                  priority_value={props.priority_value.clone()}
                  on_priority_change={props.on_priority_change.clone()}
                  due_value={props.due_value.clone()}
                  on_due_change={props.on_due_change.clone()}
                  on_clear_filters={props.on_clear_filters.clone()}
              />
          </div>
      </>
  }
}
