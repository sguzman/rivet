use yew::{
  Callback,
  Html,
  MouseEvent,
  Properties,
  function_component,
  html
};

use super::{
  BulkActions,
  GlobalActions
};

#[derive(Properties, PartialEq)]
pub struct WorkspaceActionsProps {
  pub bulk_count:         usize,
  pub on_bulk_done:
    Callback<MouseEvent>,
  pub on_bulk_delete:
    Callback<MouseEvent>,
  pub on_add_click:
    Callback<MouseEvent>,
  pub on_toggle_theme:
    Callback<MouseEvent>,
  pub theme_toggle_label: String
}

#[function_component(WorkspaceActions)]
pub fn workspace_actions(
  props: &WorkspaceActionsProps
) -> Html {
  html! {
      <div class="workspace-actions">
          <BulkActions
              bulk_count={props.bulk_count}
              on_bulk_done={props.on_bulk_done.clone()}
              on_bulk_delete={props.on_bulk_delete.clone()}
          />
          <GlobalActions
              on_add_click={props.on_add_click.clone()}
              on_toggle_theme={props.on_toggle_theme.clone()}
              theme_toggle_label={props.theme_toggle_label.clone()}
          />
      </div>
  }
}
