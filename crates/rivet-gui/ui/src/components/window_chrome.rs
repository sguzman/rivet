use yew::{
  Callback,
  Html,
  MouseEvent,
  Properties,
  function_component,
  html
};

use super::{
  WindowBrand,
  WindowControls
};

#[derive(Properties, PartialEq)]
pub struct WindowChromeProps {
  pub on_window_minimize:
    Callback<MouseEvent>,
  pub on_window_toggle_maximize:
    Callback<MouseEvent>,
  pub on_window_close:
    Callback<MouseEvent>,
  pub title:                     String,
  pub icon_src:                  String,
  pub icon_alt:                  String
}

#[function_component(WindowChrome)]
pub fn window_chrome(
  props: &WindowChromeProps
) -> Html {
  html! {
      <div class="window-chrome" data-tauri-drag-region="true">
          <WindowBrand
              title={props.title.clone()}
              icon_src={props.icon_src.clone()}
              icon_alt={props.icon_alt.clone()}
          />
          <WindowControls
              on_window_minimize={props.on_window_minimize.clone()}
              on_window_toggle_maximize={props.on_window_toggle_maximize.clone()}
              on_window_close={props.on_window_close.clone()}
          />
      </div>
  }
}
