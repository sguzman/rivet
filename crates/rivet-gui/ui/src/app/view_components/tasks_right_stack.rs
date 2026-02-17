#[derive(Properties, PartialEq)]
struct TasksRightStackProps {
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
    Callback<MouseEvent>,
  selected_task:        Option<TaskDto>,
  tag_colors:
    BTreeMap<String, String>,
  on_done:              Callback<Uuid>,
  on_delete:            Callback<Uuid>,
  on_edit:              Callback<TaskDto>
}

#[function_component(TasksRightStack)]
fn tasks_right_stack(
  props: &TasksRightStackProps
) -> Html {
  html! {
      <div class="right-stack">
          <TaskFiltersPanel
              title={"Task Filters".to_string()}
              show_search={true}
              search_value={props.search_value.clone()}
              on_search_input={props.on_search_input.clone()}
              completion_value={props.completion_value.clone()}
              on_completion_change={props.on_completion_change.clone()}
              project_value={props.project_value.clone()}
              project_items={props.project_items.clone()}
              on_project_change={props.on_project_change.clone()}
              tag_value={props.tag_value.clone()}
              tag_items={props.tag_items.clone()}
              on_tag_change={props.on_tag_change.clone()}
              priority_value={props.priority_value.clone()}
              on_priority_change={props.on_priority_change.clone()}
              due_value={props.due_value.clone()}
              on_due_change={props.on_due_change.clone()}
              on_clear_filters={props.on_clear_filters.clone()}
          />
          <Details
              task={props.selected_task.clone()}
              tag_colors={props.tag_colors.clone()}
              on_done={props.on_done.clone()}
              on_delete={props.on_delete.clone()}
              on_edit={props.on_edit.clone()}
          />
      </div>
  }
}
