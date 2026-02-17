use yew::{
  Callback,
  Html,
  MouseEvent,
  Properties,
  function_component,
  html
};

use super::WorkspaceTabButton;

#[derive(Properties, PartialEq)]
pub struct WorkspaceTabListProps {
  pub active_tab:             String,
  pub on_select_tasks_tab:
    Callback<MouseEvent>,
  pub on_select_kanban_tab:
    Callback<MouseEvent>,
  pub on_select_calendar_tab:
    Callback<MouseEvent>
}

#[function_component(WorkspaceTabList)]
pub fn workspace_tab_list(
  props: &WorkspaceTabListProps
) -> Html {
  html! {
      <div class="workspace-tab-list">
          <WorkspaceTabButton
              label={"Tasks".to_string()}
              is_active={props.active_tab == "tasks"}
              onclick={props.on_select_tasks_tab.clone()}
          />
          <WorkspaceTabButton
              label={"Kanban".to_string()}
              is_active={props.active_tab == "kanban"}
              onclick={props.on_select_kanban_tab.clone()}
          />
          <WorkspaceTabButton
              label={"Calendar".to_string()}
              is_active={props.active_tab == "calendar"}
              onclick={props.on_select_calendar_tab.clone()}
          />
      </div>
  }
}
