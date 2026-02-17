#[derive(Properties, PartialEq)]
struct KanbanFiltersPanelProps {
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

#[function_component(KanbanFiltersPanel)]
fn kanban_filters_panel(
  props: &KanbanFiltersPanelProps
) -> Html {
  html! {
      <>
          <TaskFiltersPanel
              title={"Kanban Filters".to_string()}
              show_search={false}
              search_value={String::new()}
              on_search_input={Callback::from(|_e: web_sys::InputEvent| ())}
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
      </>
  }
}
