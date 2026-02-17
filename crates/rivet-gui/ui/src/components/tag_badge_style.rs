use std::collections::BTreeMap;

pub fn tag_badge_style(
  tag: &str,
  tag_colors: &BTreeMap<String, String>
) -> String {
  if let Some((key, _)) =
    tag.split_once(':')
    && let Some(color) =
      tag_colors.get(key)
  {
    return format!(
      "--tag-key-color:{color};"
    );
  }

  String::new()
}
