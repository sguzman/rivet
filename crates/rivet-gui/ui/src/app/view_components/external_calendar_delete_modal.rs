#[derive(Properties, PartialEq)]
struct ExternalCalendarDeleteModalProps {
  modal_state:
    UseStateHandle<Option<ExternalCalendarDeleteState>>,
  on_close:
    Callback<MouseEvent>,
  on_confirm:
    Callback<String>
}

#[function_component(
  ExternalCalendarDeleteModal
)]
fn external_calendar_delete_modal(
  props: &ExternalCalendarDeleteModalProps
) -> Html {
  let delete_modal =
    props.modal_state.clone();
  let Some(state) =
    (*delete_modal).clone()
  else {
    return html! {};
  };

  let calendar_id = state.id.clone();
  let on_confirm = {
    let on_confirm = props.on_confirm.clone();
    Callback::from(move |_| {
      on_confirm.emit(calendar_id.clone())
    })
  };

  html! {
      <div class="modal-backdrop" onclick={props.on_close.clone()}>
          <div class="modal modal-sm calendar-delete-modal" onclick={Callback::from(|e: yew::MouseEvent| e.stop_propagation())}>
              <div class="header">{ "Delete External Calendar" }</div>
              <div class="content">
                  <div>
                      { format!("Delete external calendar '{}'?", state.name) }
                  </div>
                  <div class="field-help">
                      { "This removes the source from sync. Calendar-managed tasks will be removed on next sync cycle." }
                  </div>
              </div>
              <div class="footer">
                  <button
                      class="btn"
                      type="button"
                      onclick={props.on_close.clone()}
                  >
                      { "Cancel" }
                  </button>
                  <button
                      class="btn danger"
                      type="button"
                      onclick={on_confirm}
                  >
                      { "Delete" }
                  </button>
              </div>
          </div>
      </div>
  }
}
