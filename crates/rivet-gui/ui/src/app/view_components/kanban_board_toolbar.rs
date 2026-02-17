#[derive(Properties, PartialEq)]
struct KanbanBoardToolbarProps {
  board_selected:           bool,
  compact_cards:            bool,
  on_create_kanban_board:   Callback<MouseEvent>,
  on_open_rename_kanban_board:
    Callback<MouseEvent>,
  on_delete_kanban_board:   Callback<MouseEvent>,
  on_toggle_card_density:   Callback<MouseEvent>
}

#[function_component(KanbanBoardToolbar)]
fn kanban_board_toolbar(
  props: &KanbanBoardToolbarProps
) -> Html {
  html! {
      <div class="actions">
          <button class="btn" onclick={props.on_create_kanban_board.clone()}>{ "New Board" }</button>
          <button class="btn" onclick={props.on_open_rename_kanban_board.clone()} disabled={!props.board_selected}>{ "Rename" }</button>
          <button class="btn danger" onclick={props.on_delete_kanban_board.clone()} disabled={!props.board_selected}>{ "Delete" }</button>
          <button class="btn" onclick={props.on_toggle_card_density.clone()}>
              { if props.compact_cards { "Full Cards" } else { "Compact Cards" } }
          </button>
      </div>
  }
}
