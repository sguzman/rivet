use yew::{
  Html,
  Properties,
  function_component,
  html
};

#[derive(Properties, PartialEq)]
pub struct WindowBrandProps {
  pub title:    String,
  pub icon_src: String,
  pub icon_alt: String
}

#[function_component(WindowBrand)]
pub fn window_brand(
  props: &WindowBrandProps
) -> Html {
  html! {
      <div class="window-brand">
          <img class="window-mascot" src={props.icon_src.clone()} alt={props.icon_alt.clone()} />
          <span>{ props.title.clone() }</span>
      </div>
  }
}
