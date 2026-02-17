#[derive(Properties, PartialEq)]
struct TaskFiltersPanelProps {
  title:                String,
  show_search:          bool,
  search_value:         String,
  on_search_input:
    Callback<web_sys::InputEvent>,
  completion_value:     String,
  on_completion_change:
    Callback<web_sys::Event>,
  project_value:        String,
  project_items:        Vec<(String, usize)>,
  on_project_change:
    Callback<web_sys::Event>,
  tag_value:            String,
  tag_items:            Vec<(String, usize)>,
  on_tag_change:
    Callback<web_sys::Event>,
  priority_value:       String,
  on_priority_change:
    Callback<web_sys::Event>,
  due_value:            String,
  on_due_change:
    Callback<web_sys::Event>,
  on_clear_filters:
    Callback<MouseEvent>
}

#[function_component(TaskFiltersPanel)]
fn task_filters_panel(
  props: &TaskFiltersPanelProps
) -> Html {
  html! {
      <div class="panel">
          <div class="header">{ props.title.clone() }</div>
          <div class="details">
              {
                  if props.show_search {
                      html! {
                          <div class="field">
                              <label>{ "Search Tasks" }</label>
                              <input
                                  value={props.search_value.clone()}
                                  placeholder="Search tasks"
                                  oninput={props.on_search_input.clone()}
                              />
                          </div>
                      }
                  } else {
                      html! {}
                  }
              }
              <div class="field">
                  <label>{ "Completion" }</label>
                  <select
                      class="tag-select"
                      value={props.completion_value.clone()}
                      onchange={props.on_completion_change.clone()}
                  >
                      <option value="all">{ "All" }</option>
                      <option value="open">{ "Open (Pending + Waiting)" }</option>
                      <option value="pending">{ "Pending" }</option>
                      <option value="waiting">{ "Waiting" }</option>
                      <option value="completed">{ "Completed" }</option>
                      <option value="deleted">{ "Deleted" }</option>
                  </select>
              </div>
              <div class="field">
                  <label>{ "Project" }</label>
                  <select
                      class="tag-select"
                      value={props.project_value.clone()}
                      onchange={props.on_project_change.clone()}
                  >
                      <option value="">{ "All Projects" }</option>
                      {
                          for props.project_items.iter().map(|(project, count)| html! {
                              <option value={project.clone()}>{ format!("{project} ({count})") }</option>
                          })
                      }
                  </select>
              </div>
              <div class="field">
                  <label>{ "Tag" }</label>
                  <select
                      class="tag-select"
                      value={props.tag_value.clone()}
                      onchange={props.on_tag_change.clone()}
                  >
                      <option value="">{ "All Tags" }</option>
                      {
                          for props.tag_items.iter().map(|(tag, count)| html! {
                              <option value={tag.clone()}>{ format!("{tag} ({count})") }</option>
                          })
                      }
                  </select>
              </div>
              <div class="field">
                  <label>{ "Priority" }</label>
                  <select
                      class="tag-select"
                      value={props.priority_value.clone()}
                      onchange={props.on_priority_change.clone()}
                  >
                      <option value="all">{ "All Priorities" }</option>
                      <option value="low">{ "Low" }</option>
                      <option value="medium">{ "Medium" }</option>
                      <option value="high">{ "High" }</option>
                      <option value="none">{ "None" }</option>
                  </select>
              </div>
              <div class="field">
                  <label>{ "Due" }</label>
                  <select
                      class="tag-select"
                      value={props.due_value.clone()}
                      onchange={props.on_due_change.clone()}
                  >
                      <option value="all">{ "All" }</option>
                      <option value="has_due">{ "Has Due Date" }</option>
                      <option value="no_due">{ "No Due Date" }</option>
                  </select>
              </div>
              <div class="actions">
                  <button class="btn" onclick={props.on_clear_filters.clone()}>{ "Clear Filters" }</button>
              </div>
          </div>
      </div>
  }
}
