#[derive(Properties, PartialEq)]
struct KanbanBoardsSidebarProps {
  boards:                    Vec<KanbanBoardDef>,
  active_board_id:           Option<String>,
  compact_cards:             bool,
  on_create_kanban_board:    Callback<MouseEvent>,
  on_open_rename_kanban_board:
    Callback<MouseEvent>,
  on_delete_kanban_board:    Callback<MouseEvent>,
  on_toggle_card_density:    Callback<MouseEvent>,
  on_select_board_id:        Callback<String>
}

#[function_component(KanbanBoardsSidebar)]
fn kanban_boards_sidebar(
  props: &KanbanBoardsSidebarProps
) -> Html {
  html! {
      <div class="panel board-sidebar">
          <div class="header">{ "Kanban Boards" }</div>
          <div class="details">
              <KanbanBoardToolbar
                  board_selected={props.active_board_id.is_some()}
                  compact_cards={props.compact_cards}
                  on_create_kanban_board={props.on_create_kanban_board.clone()}
                  on_open_rename_kanban_board={props.on_open_rename_kanban_board.clone()}
                  on_delete_kanban_board={props.on_delete_kanban_board.clone()}
                  on_toggle_card_density={props.on_toggle_card_density.clone()}
              />
              <KanbanBoardList
                  boards={props.boards.clone()}
                  active_board_id={props.active_board_id.clone()}
                  on_select_board_id={props.on_select_board_id.clone()}
              />
          </div>
      </div>
  }
}
