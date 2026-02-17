fn render_calendar_markers(
  markers: &[CalendarTaskMarker],
  limit: usize
) -> Html {
  if markers.is_empty() {
    return html! {};
  }

  let capped = markers.len().min(limit);
  let overflow = markers
    .len()
    .saturating_sub(capped);

  html! {
      <div class="calendar-markers">
          {
              for markers.iter().take(capped).map(|marker| {
                  let style = format!("--marker-color:{};", marker.color);
                  html! {
                      <span class={classes!("calendar-marker", marker.shape.as_class())} style={style}></span>
                  }
              })
          }
          {
              if overflow > 0 {
                  html! { <span class="badge">{ format!("+{overflow}") }</span> }
              } else {
                  html! {}
              }
          }
      </div>
  }
}

