#[derive(Properties, PartialEq)]
struct NewKanbanBoardModalProps {
  open:                  bool,
  input_value:           String,
  on_close:              Callback<MouseEvent>,
  on_input:              Callback<web_sys::InputEvent>,
  on_submit:             Callback<MouseEvent>
}

#[function_component(NewKanbanBoardModal)]
fn new_kanban_board_modal(
  props: &NewKanbanBoardModalProps
) -> Html {
  if !props.open {
    return html! {};
  }

  html! {
      <div class="modal-backdrop" onclick={props.on_close.clone()}>
          <div class="modal modal-sm" onclick={Callback::from(|e: yew::MouseEvent| e.stop_propagation())}>
              <div class="header">{ "New Kanban Board" }</div>
              <div class="content">
                  <div class="field">
                      <label>{ "Board Name" }</label>
                      <input
                          value={props.input_value.clone()}
                          oninput={props.on_input.clone()}
                          placeholder="Board name"
                      />
                  </div>
                  <div class="footer">
                      <button type="button" class="btn" onclick={props.on_close.clone()}>{ "Cancel" }</button>
                      <button
                          type="button"
                          class="btn"
                          onclick={props.on_submit.clone()}
                          disabled={props.input_value.trim().is_empty()}
                      >
                          { "Create" }
                      </button>
                  </div>
              </div>
          </div>
      </div>
  }
}
