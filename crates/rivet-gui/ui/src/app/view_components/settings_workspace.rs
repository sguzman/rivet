#[derive(Properties, PartialEq)]
struct SettingsWorkspaceProps {
  active_view: String,
  on_nav:      Callback<String>,
  tasks_loaded: usize,
  bulk_count:  usize
}

#[function_component(SettingsWorkspace)]
fn settings_workspace(
  props: &SettingsWorkspaceProps
) -> Html {
  html! {
      <>
          <Sidebar active={props.active_view.clone()} on_nav={props.on_nav.clone()} />
          <div class="panel list">
              <div class="header">{ "Settings" }</div>
              <div class="details">
                  <div>{ "The desktop UI is a thin client over the core Rivet datastore." }</div>
                  <div class="kv"><strong>{ "view" }</strong><div>{ "settings" }</div></div>
                  <div class="kv"><strong>{ "status" }</strong><div>{ "core + tauri bridge active" }</div></div>
                  <div class="kv"><strong>{ "workflow" }</strong><div>{ "Use context/report commands in CLI for advanced behavior." }</div></div>
              </div>
          </div>
          <div class="panel">
              <div class="header">{ "Current Data" }</div>
              <div class="details">
                  <div class="kv"><strong>{ "tasks loaded" }</strong><div>{ props.tasks_loaded }</div></div>
                  <div class="kv"><strong>{ "selected" }</strong><div>{ props.bulk_count }</div></div>
              </div>
          </div>
      </>
  }
}
