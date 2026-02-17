#[function_component(CalendarMarkerLegend)]
fn calendar_marker_legend() -> Html {
  html! {
      <>
          <div class="calendar-dot-legend">
              <span class="calendar-marker triangle" style="--marker-color:var(--accent);"></span>
              <span>{ "Kanban board task" }</span>
          </div>
          <div class="calendar-dot-legend">
              <span class="calendar-marker circle" style="--marker-color:#d64545;"></span>
              <span>{ "External calendar task" }</span>
          </div>
          <div class="calendar-dot-legend">
              <span class="calendar-marker square" style="--marker-color:#7f8691;"></span>
              <span>{ "Unassigned task" }</span>
          </div>
      </>
  }
}
