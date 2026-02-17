use yew::{
  Callback,
  Html,
  Properties,
  function_component,
  html
};

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
  pub active: String,
  pub on_nav: Callback<String>
}

#[function_component(Sidebar)]
pub fn sidebar(
  props: &SidebarProps
) -> Html {
  let make_item =
    |key: &str, label: &str| {
      let active = props.active == key;
      let class = if active {
        "item active"
      } else {
        "item"
      };
      let on_nav = props.on_nav.clone();
      let key_string = key.to_string();
      html! {
          <div class={class} onclick={move |_| on_nav.emit(key_string.clone())}>
              { label }
          </div>
      }
    };

  html! {
      <div class="panel sidebar">
          <div class="header">{ "Views" }</div>
          { make_item("all", "Tasks") }
          { make_item("projects", "Projects") }
          { make_item("tags", "Tags") }
          { make_item("settings", "Settings") }
      </div>
  }
}
