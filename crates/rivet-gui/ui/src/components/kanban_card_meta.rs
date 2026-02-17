use std::collections::BTreeMap;

use rivet_gui_shared::TaskDto;
use yew::{
  Html,
  Properties,
  function_component,
  html
};

use super::TaskTagBadge;

#[derive(Properties, PartialEq)]
pub struct KanbanCardMetaProps {
  pub task:       TaskDto,
  pub tag_colors:
    BTreeMap<String, String>
}

#[function_component(KanbanCardMeta)]
pub fn kanban_card_meta(
  props: &KanbanCardMetaProps
) -> Html {
  html! {
      <>
          <div class="kanban-card-meta">
              <span class="badge">
                  {
                      if let Some(project) = props.task.project.clone() {
                          format!("project:{project}")
                      } else {
                          "project:—".to_string()
                      }
                  }
              </span>
              {
                  if let Some(due) = props.task.due.clone() {
                      html! { <span class="badge">{ format!("due:{due}") }</span> }
                  } else {
                      html! {}
                  }
              }
          </div>
          <div class="kanban-card-meta">
              {
                  for props.task.tags.iter().take(3).cloned().map(|tag| html! {
                      <TaskTagBadge tag={tag} tag_colors={props.tag_colors.clone()} />
                  })
              }
          </div>
      </>
  }
}
