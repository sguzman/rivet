fn weekday_labels(
  week_start: Weekday
) -> Vec<&'static str> {
  match week_start {
    | Weekday::Sun => {
      vec![
        "Sun", "Mon", "Tue", "Wed",
        "Thu", "Fri", "Sat",
      ]
    }
    | _ => {
      vec![
        "Mon", "Tue", "Wed", "Thu",
        "Fri", "Sat", "Sun",
      ]
    }
  }
}

