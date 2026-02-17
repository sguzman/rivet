#[derive(Properties, PartialEq)]
struct CalendarViewSwitchProps {
  current_view: CalendarViewMode,
  on_set_view:
    Callback<CalendarViewMode>
}

#[function_component(CalendarViewSwitch)]
fn calendar_view_switch(
  props: &CalendarViewSwitchProps
) -> Html {
  html! {
      <div class="calendar-view-switch">
          {
              for CalendarViewMode::all().iter().copied().map(|view| {
                  let on_set_view = props.on_set_view.clone();
                  let is_active = props.current_view == view;
                  html! {
                      <button
                          class={classes!("calendar-view-btn", is_active.then_some("active"))}
                          onclick={Callback::from(move |_| on_set_view.emit(view))}
                      >
                          { view.label() }
                      </button>
                  }
              })
          }
      </div>
  }
}
