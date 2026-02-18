#[derive(Properties, PartialEq)]
struct SettingsWorkspaceProps {
  active_view:        String,
  on_nav:             Callback<String>,
  tasks_loaded:       usize,
  bulk_count:         usize,
  due_notifications:  DueNotificationConfig,
  due_permission:     DueNotificationPermission,
  on_due_enabled:
    Callback<web_sys::Event>,
  on_due_pre_enabled:
    Callback<web_sys::Event>,
  on_due_pre_minutes_input:
    Callback<web_sys::InputEvent>,
  on_request_due_permission:
    Callback<MouseEvent>
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
          <div class="panel">
              <div class="header">{ "Due Notifications" }</div>
              <div class="details">
                  <div class="field field-inline-check">
                      <label>{ "Enable OS due notifications" }</label>
                      <input
                          type="checkbox"
                          checked={props.due_notifications.enabled}
                          onchange={props.on_due_enabled.clone()}
                      />
                  </div>
                  <div class="field field-inline-check">
                      <label>{ "Enable pre-notify" }</label>
                      <input
                          type="checkbox"
                          checked={props.due_notifications.pre_notify_enabled}
                          disabled={!props.due_notifications.enabled}
                          onchange={props.on_due_pre_enabled.clone()}
                      />
                  </div>
                  <div class="field">
                      <label>{ "Pre-notify minutes before due" }</label>
                      <input
                          type="number"
                          min="1"
                          max="43200"
                          value={props.due_notifications.pre_notify_minutes.to_string()}
                          disabled={!props.due_notifications.enabled || !props.due_notifications.pre_notify_enabled}
                          oninput={props.on_due_pre_minutes_input.clone()}
                      />
                  </div>
                  <div class="field-help">
                      { format!("Permission: {}", props.due_permission.as_label()) }
                  </div>
                  <div class="actions">
                      <button
                          class="btn"
                          type="button"
                          onclick={props.on_request_due_permission.clone()}
                          disabled={matches!(props.due_permission, DueNotificationPermission::Unsupported)}
                      >
                          { "Request Notification Permission" }
                      </button>
                  </div>
              </div>
          </div>
      </>
  }
}
