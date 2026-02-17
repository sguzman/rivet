use yew::{
  Callback,
  Html,
  MouseEvent,
  Properties,
  function_component,
  html
};

#[derive(Properties, PartialEq)]
pub struct WindowControlsProps {
  pub on_window_minimize:
    Callback<MouseEvent>,
  pub on_window_toggle_maximize:
    Callback<MouseEvent>,
  pub on_window_close:
    Callback<MouseEvent>
}

#[function_component(WindowControls)]
pub fn window_controls(
  props: &WindowControlsProps
) -> Html {
  html! {
      <div class="window-controls" data-tauri-drag-region="false">
          <button class="window-btn" type="button" onclick={props.on_window_minimize.clone()} title="Minimize">{ "_" }</button>
          <button class="window-btn" type="button" onclick={props.on_window_toggle_maximize.clone()} title="Maximize/Restore">{ "[ ]" }</button>
          <button class="window-btn danger" type="button" onclick={props.on_window_close.clone()} title="Close">{ "X" }</button>
      </div>
  }
}
