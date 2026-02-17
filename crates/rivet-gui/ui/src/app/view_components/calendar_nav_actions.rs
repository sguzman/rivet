#[derive(Properties, PartialEq)]
struct CalendarNavActionsProps {
  on_prev:  Callback<MouseEvent>,
  on_today: Callback<MouseEvent>,
  on_next:  Callback<MouseEvent>
}

#[function_component(CalendarNavActions)]
fn calendar_nav_actions(
  props: &CalendarNavActionsProps
) -> Html {
  html! {
      <div class="actions calendar-nav-actions">
          <button class="btn" onclick={props.on_prev.clone()}>{ "Prev" }</button>
          <button class="btn" onclick={props.on_today.clone()}>{ "Today" }</button>
          <button class="btn" onclick={props.on_next.clone()}>{ "Next" }</button>
      </div>
  }
}
