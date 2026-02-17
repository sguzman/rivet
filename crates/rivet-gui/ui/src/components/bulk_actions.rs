use yew::{
  Callback,
  Html,
  MouseEvent,
  Properties,
  function_component,
  html
};

#[derive(Properties, PartialEq)]
pub struct BulkActionsProps {
  pub bulk_count:     usize,
  pub on_bulk_done:
    Callback<MouseEvent>,
  pub on_bulk_delete:
    Callback<MouseEvent>
}

#[function_component(BulkActions)]
pub fn bulk_actions(
  props: &BulkActionsProps
) -> Html {
  if props.bulk_count == 0 {
    return html! {};
  }

  html! {
      <>
          <button class="btn ok" onclick={props.on_bulk_done.clone()}>{ format!("Done {}", props.bulk_count) }</button>
          <button class="btn danger" onclick={props.on_bulk_delete.clone()}>{ format!("Delete {}", props.bulk_count) }</button>
      </>
  }
}
