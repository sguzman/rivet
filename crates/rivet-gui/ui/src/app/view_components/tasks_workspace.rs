#[derive(Properties, PartialEq)]
struct TasksWorkspaceProps {
  active_view:             String,
  on_nav:                  Callback<String>,
  task_visible_tasks:      Vec<TaskDto>,
  tag_colors:
    BTreeMap<String, String>,
  selected:                Option<Uuid>,
  bulk_selected:           BTreeSet<Uuid>,
  on_select:               Callback<Uuid>,
  on_toggle_select:        Callback<Uuid>,
  selected_task:           Option<TaskDto>,
  active_project:          Option<String>,
  active_tag:              Option<String>,
  project_facets:          Vec<(String, usize)>,
  tag_facets:              Vec<(String, usize)>,
  on_choose_project:
    Callback<Option<String>>,
  on_choose_tag:
    Callback<Option<String>>,
  search_value:            String,
  on_search_input:
    Callback<web_sys::InputEvent>,
  completion_value:        String,
  on_completion_change:
    Callback<web_sys::Event>,
  project_value:           String,
  on_project_change:
    Callback<web_sys::Event>,
  tag_value:               String,
  on_tag_change:
    Callback<web_sys::Event>,
  priority_value:          String,
  on_priority_change:
    Callback<web_sys::Event>,
  due_value:               String,
  on_due_change:
    Callback<web_sys::Event>,
  on_clear_filters:
    Callback<MouseEvent>,
  on_done:                 Callback<Uuid>,
  on_delete:               Callback<Uuid>,
  on_edit:                 Callback<TaskDto>
}

#[function_component(TasksWorkspace)]
fn tasks_workspace(
  props: &TasksWorkspaceProps
) -> Html {
  html! {
      <>
          <Sidebar active={props.active_view.clone()} on_nav={props.on_nav.clone()} />
          <TaskList
              tasks={props.task_visible_tasks.clone()}
              tag_colors={props.tag_colors.clone()}
              selected={props.selected}
              selected_ids={props.bulk_selected.clone()}
              on_select={props.on_select.clone()}
              on_toggle_select={props.on_toggle_select.clone()}
          />
          {
              if props.active_view == "projects" && props.selected_task.is_none() {
                  html! {
                      <FacetPanel
                          title={"Projects".to_string()}
                          selected={props.active_project.clone()}
                          items={props.project_facets.clone()}
                          on_select={props.on_choose_project.clone()}
                      />
                  }
              } else if props.active_view == "all" {
                  html! {
                      <TasksRightStack
                          search_value={props.search_value.clone()}
                          on_search_input={props.on_search_input.clone()}
                          completion_value={props.completion_value.clone()}
                          on_completion_change={props.on_completion_change.clone()}
                          project_value={props.project_value.clone()}
                          project_items={props.project_facets.clone()}
                          on_project_change={props.on_project_change.clone()}
                          tag_value={props.tag_value.clone()}
                          tag_items={props.tag_facets.clone()}
                          on_tag_change={props.on_tag_change.clone()}
                          priority_value={props.priority_value.clone()}
                          on_priority_change={props.on_priority_change.clone()}
                          due_value={props.due_value.clone()}
                          on_due_change={props.on_due_change.clone()}
                          on_clear_filters={props.on_clear_filters.clone()}
                          selected_task={props.selected_task.clone()}
                          tag_colors={props.tag_colors.clone()}
                          on_done={props.on_done.clone()}
                          on_delete={props.on_delete.clone()}
                          on_edit={props.on_edit.clone()}
                      />
                  }
              } else if props.active_view == "tags" && props.selected_task.is_none() {
                  html! {
                      <FacetPanel
                          title={"Tags".to_string()}
                          selected={props.active_tag.clone()}
                          items={props.tag_facets.clone()}
                          on_select={props.on_choose_tag.clone()}
                      />
                  }
              } else {
                  html! {
                      <Details
                          task={props.selected_task.clone()}
                          tag_colors={props.tag_colors.clone()}
                          on_done={props.on_done.clone()}
                          on_delete={props.on_delete.clone()}
                          on_edit={props.on_edit.clone()}
                      />
                  }
              }
          }
      </>
  }
}
