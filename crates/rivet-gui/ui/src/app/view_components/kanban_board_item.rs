#[derive(Properties, PartialEq)]
struct KanbanBoardItemProps {
  board:              KanbanBoardDef,
  is_active:          bool,
  on_select_board_id:
    Callback<String>
}

#[function_component(KanbanBoardItem)]
fn kanban_board_item(
  props: &KanbanBoardItemProps
) -> Html {
  let board_id = props.board.id.clone();
  let board_label = props.board.name.clone();
  let board_color_style =
    format!(
      "background:{};",
      props.board.color
    );
  let class = if props.is_active {
    "board-item active"
  } else {
    "board-item"
  };

  html! {
      <div class={class} onclick={{
          let on_select_board_id = props.on_select_board_id.clone();
          Callback::from(move |_| on_select_board_id.emit(board_id.clone()))
      }}>
          <div class="board-item-line">
              <span class="board-color-dot" style={board_color_style}></span>
              <span>{ board_label }</span>
          </div>
      </div>
  }
}
