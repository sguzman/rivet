use std::collections::BTreeMap;

use yew::{
  Html,
  Properties,
  function_component,
  html
};

use super::tag_badge_style::tag_badge_style;

#[derive(Properties, PartialEq)]
pub struct TaskTagBadgeProps {
  pub tag:        String,
  pub tag_colors:
    BTreeMap<String, String>
}

#[function_component(TaskTagBadge)]
pub fn task_tag_badge(
  props: &TaskTagBadgeProps
) -> Html {
  html! {
      <span class="badge tag-badge" style={tag_badge_style(&props.tag, &props.tag_colors)}>{ format!("#{}", props.tag) }</span>
  }
}
