#[derive(Properties, PartialEq)]
struct CalendarStatsPanelProps {
  stats: CalendarStats
}

#[function_component(CalendarStatsPanel)]
fn calendar_stats_panel(
  props: &CalendarStatsPanelProps
) -> Html {
  html! {
      <div class="panel">
          <div class="header">{ "Calendar Stats" }</div>
          <div class="details">
              <div class="kv"><strong>{ "period tasks" }</strong><div>{ props.stats.total }</div></div>
              <div class="kv"><strong>{ "pending" }</strong><div>{ props.stats.pending }</div></div>
              <div class="kv"><strong>{ "waiting" }</strong><div>{ props.stats.waiting }</div></div>
              <div class="kv"><strong>{ "completed" }</strong><div>{ props.stats.completed }</div></div>
              <div class="kv"><strong>{ "deleted" }</strong><div>{ props.stats.deleted }</div></div>
          </div>
      </div>
  }
}
