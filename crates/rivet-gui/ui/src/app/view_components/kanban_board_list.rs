#[derive(Properties, PartialEq)]
struct KanbanBoardListProps {
  boards:             Vec<KanbanBoardDef>,
  active_board_id:    Option<String>,
  on_select_board_id: Callback<String>
}

#[function_component(KanbanBoardList)]
fn kanban_board_list(
  props: &KanbanBoardListProps
) -> Html {
  if props.boards.is_empty() {
    return html! { <div style="color:var(--muted);">{ "No boards yet. Create one to begin." }</div> };
  }

  html! {
      <div class="board-list">
          {
              for props.boards.iter().cloned().map(|board| {
                  let is_active = props.active_board_id.as_deref() == Some(board.id.as_str());
                  html! {
                      <KanbanBoardItem
                          board={board}
                          is_active={is_active}
                          on_select_board_id={props.on_select_board_id.clone()}
                      />
                  }
              })
          }
      </div>
  }
}
