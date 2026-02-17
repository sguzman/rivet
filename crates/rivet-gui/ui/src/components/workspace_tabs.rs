use yew::{
  Callback,
  Html,
  MouseEvent,
  Properties,
  function_component,
  html
};

use super::{
  WorkspaceActions,
  WorkspaceTabList
};

#[derive(Properties, PartialEq)]
pub struct WorkspaceTabsProps {
  pub active_tab:             String,
  pub on_select_tasks_tab:
    Callback<MouseEvent>,
  pub on_select_kanban_tab:
    Callback<MouseEvent>,
  pub on_select_calendar_tab:
    Callback<MouseEvent>,
  pub bulk_count:             usize,
  pub on_bulk_done:
    Callback<MouseEvent>,
  pub on_bulk_delete:
    Callback<MouseEvent>,
  pub on_add_click:
    Callback<MouseEvent>,
  pub on_toggle_theme:
    Callback<MouseEvent>,
  pub theme_toggle_label:     String
}

#[function_component(WorkspaceTabs)]
pub fn workspace_tabs(
  props: &WorkspaceTabsProps
) -> Html {
  html! {
      <div class="workspace-tabs">
          <WorkspaceTabList
              active_tab={props.active_tab.clone()}
              on_select_tasks_tab={props.on_select_tasks_tab.clone()}
              on_select_kanban_tab={props.on_select_kanban_tab.clone()}
              on_select_calendar_tab={props.on_select_calendar_tab.clone()}
          />
          <WorkspaceActions
              bulk_count={props.bulk_count}
              on_bulk_done={props.on_bulk_done.clone()}
              on_bulk_delete={props.on_bulk_delete.clone()}
              on_add_click={props.on_add_click.clone()}
              on_toggle_theme={props.on_toggle_theme.clone()}
              theme_toggle_label={props.theme_toggle_label.clone()}
          />
      </div>
  }
}
