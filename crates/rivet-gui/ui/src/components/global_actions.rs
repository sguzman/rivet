use yew::{
  Callback,
  Html,
  MouseEvent,
  Properties,
  function_component,
  html
};

#[derive(Properties, PartialEq)]
pub struct GlobalActionsProps {
  pub on_add_click:
    Callback<MouseEvent>,
  pub on_toggle_theme:
    Callback<MouseEvent>,
  pub theme_toggle_label: String
}

#[function_component(GlobalActions)]
pub fn global_actions(
  props: &GlobalActionsProps
) -> Html {
  html! {
      <>
          <button class="btn" onclick={props.on_add_click.clone()}>{ "Add Task" }</button>
          <button class="btn" onclick={props.on_toggle_theme.clone()}>{ props.theme_toggle_label.clone() }</button>
      </>
  }
}
