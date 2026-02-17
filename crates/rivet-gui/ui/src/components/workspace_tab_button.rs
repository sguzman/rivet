use yew::{
  Callback,
  Html,
  MouseEvent,
  Properties,
  function_component,
  html
};

#[derive(Properties, PartialEq)]
pub struct WorkspaceTabButtonProps {
  pub label:     String,
  pub is_active: bool,
  pub onclick:   Callback<MouseEvent>
}

#[function_component(
  WorkspaceTabButton
)]
pub fn workspace_tab_button(
  props: &WorkspaceTabButtonProps
) -> Html {
  html! {
      <button
          class={if props.is_active { "workspace-tab active" } else { "workspace-tab" }}
          onclick={props.onclick.clone()}
      >
          { props.label.clone() }
      </button>
  }
}
