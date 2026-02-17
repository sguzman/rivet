use yew::{
  Callback,
  Html,
  Properties,
  function_component,
  html
};

#[derive(Properties, PartialEq)]
pub struct FacetPanelProps {
  pub title:     String,
  pub items:     Vec<(String, usize)>,
  pub selected:  Option<String>,
  pub on_select:
    Callback<Option<String>>
}

#[function_component(FacetPanel)]
pub fn facet_panel(
  props: &FacetPanelProps
) -> Html {
  let on_select_all =
    props.on_select.clone();

  html! {
      <div class="panel">
          <div class="header">{ &props.title }</div>
          <div class="details">
              <div
                  class={if props.selected.is_none() { "facet active" } else { "facet" }}
                  onclick={move |_| on_select_all.emit(None)}
              >
                  <span>{ "All" }</span>
              </div>

              {
                  for props.items.iter().map(|(item, count)| {
                      let item_name = item.clone();
                      let on_select = props.on_select.clone();
                      let is_active = props.selected.as_deref() == Some(item.as_str());
                      let class = if is_active { "facet active" } else { "facet" };
                      html! {
                          <div class={class} onclick={move |_| on_select.emit(Some(item_name.clone()))}>
                              <span>{ item }</span>
                              <span class="badge">{ *count }</span>
                          </div>
                      }
                  })
              }
          </div>
      </div>
  }
}
