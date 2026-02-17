#[derive(Properties, PartialEq)]
struct CalendarMetaSummaryProps {
  timezone:   Tz,
  focus_date: NaiveDate,
  due_count:  usize
}

#[function_component(CalendarMetaSummary)]
fn calendar_meta_summary(
  props: &CalendarMetaSummaryProps
) -> Html {
  html! {
      <>
          <div class="kv">
              <strong>{ "timezone" }</strong>
              <div>{ props.timezone.to_string() }</div>
          </div>
          <div class="kv">
              <strong>{ "focus date" }</strong>
              <div>{ props.focus_date.format("%Y-%m-%d").to_string() }</div>
          </div>
          <div class="kv">
              <strong>{ "due tasks" }</strong>
              <div>{ props.due_count }</div>
          </div>
      </>
  }
}
