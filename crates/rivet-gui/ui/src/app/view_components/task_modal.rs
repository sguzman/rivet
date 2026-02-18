#[derive(Properties, PartialEq)]
struct TaskModalProps {
  modal_state:
    UseStateHandle<Option<ModalState>>,
  modal_busy:           bool,
  kanban_boards:        Vec<KanbanBoardDef>,
  tag_schema:           TagSchema,
  tag_colors:
    BTreeMap<String, String>,
  on_modal_submit:
    Callback<ModalState>,
  on_modal_close_click:
    Callback<MouseEvent>
}

#[function_component(TaskModal)]
fn task_modal(
  props: &TaskModalProps
) -> Html {
  let modal_state =
    props.modal_state.clone();
  let modal_busy = props.modal_busy;
  let kanban_boards =
    props.kanban_boards.clone();
  let tag_schema =
    props.tag_schema.clone();
  let on_modal_submit =
    props.on_modal_submit.clone();
  let on_modal_close_click =
    props.on_modal_close_click.clone();
  let tag_colors =
    props.tag_colors.clone();
              if let Some(state) = (*modal_state).clone() {
                  let submit_state = state.clone();
                  let is_busy = modal_busy;
                  let board_options: Vec<(String, String)> = kanban_boards
                      .iter()
                      .map(|board| (board.id.clone(), board.name.clone()))
                      .collect();
                  let selected_board_name = state
                      .draft_board_id
                      .as_ref()
                      .and_then(|board_id| {
                          board_options
                              .iter()
                              .find(|(id, _name)| id == board_id)
                              .map(|(_id, name)| name.clone())
                      });
                  let picker_value_options = state
                      .picker_key
                      .as_deref()
                      .and_then(|id| tag_schema.key(id))
                      .map(|key| key.values.clone())
                      .unwrap_or_default();
                  let on_save_click = {
                      let on_modal_submit = on_modal_submit.clone();
                      let submit_state = submit_state.clone();
                      Callback::from(move |_| {
                          ui_debug("button.save.click", "save click fired");
                          on_modal_submit.emit(submit_state.clone());
                      })
                  };
                  let on_add_custom_tag = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |_| {
                          if let Some(mut current) = (*modal_state).clone() {
                              let custom_tags = split_tags(&current.draft_custom_tag);
                              if custom_tags.is_empty() {
                                  return;
                              }

                              let mut added = 0_usize;
                              for tag in custom_tags {
                                  if push_tag_unique(&mut current.draft_tags, tag) {
                                      added += 1;
                                  }
                              }
                              tracing::debug!(added, "added custom tags");
                              current.draft_custom_tag.clear();
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_picker_key_change = {
                      let modal_state = modal_state.clone();
                      let tag_schema = tag_schema.clone();
                      Callback::from(move |e: web_sys::Event| {
                          let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                          let key = select.value();
                          if let Some(mut current) = (*modal_state).clone() {
                              if key.trim().is_empty() {
                                  current.picker_key = None;
                                  current.picker_value = None;
                              } else {
                                  current.picker_key = Some(key.clone());
                                  current.picker_value = first_value_for_key(&tag_schema, &key);
                              }
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_picker_value_change = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |e: web_sys::Event| {
                          let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                          let value = select.value();
                          if let Some(mut current) = (*modal_state).clone() {
                              current.picker_value = if value.trim().is_empty() {
                                  None
                              } else {
                                  Some(value)
                              };
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_add_picker_tag = {
                      let modal_state = modal_state.clone();
                      let tag_schema = tag_schema.clone();
                      Callback::from(move |_| {
                          if let Some(mut current) = (*modal_state).clone() {
                              let Some(key) = current.picker_key.clone() else {
                                  return;
                              };
                              let Some(value) = current.picker_value.clone() else {
                                  return;
                              };

                              let key = key.trim();
                              let value = value.trim();
                              if key.is_empty() || value.is_empty() {
                                  return;
                              }

                              let tag = format!("{key}:{value}");
                              if is_single_select_key(&tag_schema, key) {
                                  remove_tags_for_key(&mut current.draft_tags, key);
                              }
                              if push_tag_unique(&mut current.draft_tags, tag.clone()) {
                                  tracing::debug!(tag = %tag, "added picker tag");
                              } else {
                                  tracing::debug!(tag = %tag, "picker tag already present");
                              }

                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_board_change = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |e: web_sys::Event| {
                          let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                          let value = select.value();
                          if let Some(mut current) = (*modal_state).clone() {
                              current.draft_board_id = if value.trim().is_empty() {
                                  None
                              } else {
                                  Some(value)
                              };
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_recurrence_pattern_change = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |e: web_sys::Event| {
                          let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                          let value = select.value();
                          if let Some(mut current) = (*modal_state).clone() {
                              current.recurrence_pattern = value;
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_recurrence_time_change = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |e: web_sys::InputEvent| {
                          let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                          if let Some(mut current) = (*modal_state).clone() {
                              current.recurrence_time = input.value();
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  let on_recurrence_month_day_change = {
                      let modal_state = modal_state.clone();
                      Callback::from(move |e: web_sys::InputEvent| {
                          let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                          if let Some(mut current) = (*modal_state).clone() {
                              current.recurrence_month_day = input.value();
                              current.error = None;
                              modal_state.set(Some(current));
                          }
                      })
                  };
                  html! {
                      <div class="modal-backdrop">
                          <div class="modal">
                              <div class="header">
                                  {
                                      match state.mode {
                                          ModalMode::Add => "Add Task",
                                          ModalMode::Edit(_) => "Edit Task",
                                      }
                                  }
                              </div>
                              <div class="content">
                                  {
                                      if let Some(err) = state.error.clone() {
                                          html! { <div class="form-error">{ err }</div> }
                                      } else {
                                          html! {}
                                      }
                                  }
                                  <div class="field">
                                      <label>{ "Title" }</label>
                                      <input
                                          value={state.draft_title.clone()}
                                          placeholder="Required task title"
                                          oninput={{
                                              let modal_state = modal_state.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*modal_state).clone() {
                                                      current.draft_title = input.value();
                                                      current.error = None;
                                                      modal_state.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>{ "Description (optional)" }</label>
                                      <input
                                          value={state.draft_desc.clone()}
                                          placeholder="Optional details"
                                          oninput={{
                                              let modal_state = modal_state.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*modal_state).clone() {
                                                      current.draft_desc = input.value();
                                                      current.error = None;
                                                      modal_state.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>{ "Project" }</label>
                                      <input
                                          value={state.draft_project.clone()}
                                          oninput={{
                                              let modal_state = modal_state.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*modal_state).clone() {
                                                      current.draft_project = input.value();
                                                      current.error = None;
                                                      modal_state.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  <div class="field">
                                      <label>
                                          {
                                              if state.lock_board_selection {
                                                  "Kanban Board (fixed by current board)"
                                              } else {
                                                  "Kanban Board (optional)"
                                              }
                                          }
                                      </label>
                                      <select
                                          class="tag-select"
                                          value={state.draft_board_id.clone().unwrap_or_default()}
                                          onchange={on_board_change}
                                          disabled={state.lock_board_selection}
                                      >
                                          <option value="">{ "No board (won't appear on Kanban)" }</option>
                                          {
                                              for board_options.iter().map(|(board_id, board_name)| html! {
                                                  <option value={board_id.clone()}>{ board_name.clone() }</option>
                                              })
                                          }
                                      </select>
                                      {
                                          if state.lock_board_selection {
                                              html! {
                                                  <div class="field-help">
                                                      {
                                                          selected_board_name
                                                              .map(|name| format!("This task will be added to board: {name}"))
                                                              .unwrap_or_else(|| "This task will be added to the active board.".to_string())
                                                      }
                                                  </div>
                                              }
                                          } else {
                                              html! {}
                                          }
                                      }
                                  </div>
                                  <div class="field">
                                      <label>{ "Custom Tag" }</label>
                                      <div class="field-inline">
                                          <input
                                              value={state.draft_custom_tag.clone()}
                                              placeholder="e.g. topic:corn or followup"
                                              oninput={{
                                                  let modal_state = modal_state.clone();
                                                  Callback::from(move |e: web_sys::InputEvent| {
                                                      let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                      if let Some(mut current) = (*modal_state).clone() {
                                                          current.draft_custom_tag = input.value();
                                                          current.error = None;
                                                          modal_state.set(Some(current));
                                                      }
                                                  })
                                              }}
                                          />
                                          <button
                                              type="button"
                                              class="btn"
                                              onclick={on_add_custom_tag}
                                          >
                                              { "Add" }
                                          </button>
                                      </div>
                                  </div>
                                  <div class="field">
                                      <label>{ "Pick Tag (key -> value)" }</label>
                                      <div class="tag-picker">
                                          <select
                                              class="tag-select"
                                              value={state.picker_key.clone().unwrap_or_default()}
                                              onchange={on_picker_key_change}
                                          >
                                              <option value="">{ "Select key" }</option>
                                              {
                                                  for tag_schema.keys.iter().filter(|key| key.id != BOARD_TAG_KEY).map(|key| {
                                                      let label = key.label.clone().unwrap_or_else(|| key.id.clone());
                                                      html! {
                                                          <option value={key.id.clone()}>
                                                              { format!("{label} ({})", key.id) }
                                                          </option>
                                                      }
                                                  })
                                              }
                                          </select>
                                          <select
                                              class="tag-select"
                                              value={state.picker_value.clone().unwrap_or_default()}
                                              onchange={on_picker_value_change}
                                              disabled={state.picker_key.is_none() || picker_value_options.is_empty()}
                                          >
                                              <option value="">{ "Select value" }</option>
                                              {
                                                  for picker_value_options.iter().map(|value| html! {
                                                      <option value={value.clone()}>{ value }</option>
                                                  })
                                              }
                                          </select>
                                          <button
                                              type="button"
                                              class="btn tag-plus"
                                              onclick={on_add_picker_tag}
                                              disabled={state.picker_key.is_none() || state.picker_value.is_none()}
                                              title="Add selected key:value tag"
                                          >
                                              { "+" }
                                          </button>
                                      </div>
                                  </div>
                                  <div class="field">
                                      <label>{ "Selected Tags" }</label>
                                      <div class="tag-list">
                                          {
                                              if state.draft_tags.is_empty() {
                                                  html! { <span class="tag-empty">{ "No tags selected yet." }</span> }
                                              } else {
                                                  html! {
                                                      <>
                                                          {
                                                              for state.draft_tags.iter().map(|tag| {
                                                                  let modal_state = modal_state.clone();
                                                                  let tag_to_remove = tag.clone();
                                                                  let chip_style = tag_chip_style(&tag_schema, tag, &tag_colors);
                                                                  html! {
                                                                      <span class="tag-chip" style={chip_style}>
                                                                          <span>{ tag }</span>
                                                                          <button
                                                                              type="button"
                                                                              class="tag-chip-remove"
                                                                              onclick={Callback::from(move |_| {
                                                                                  if let Some(mut current) = (*modal_state).clone() {
                                                                                      current.draft_tags.retain(|value| value != &tag_to_remove);
                                                                                      current.error = None;
                                                                                      modal_state.set(Some(current));
                                                                                  }
                                                                              })}
                                                                          >
                                                                              { "x" }
                                                                          </button>
                                                                      </span>
                                                                  }
                                                              })
                                                          }
                                                      </>
                                                  }
                                              }
                                          }
                                      </div>
                                  </div>
                                  <div class="field">
                                      <label>{ "Due" }</label>
                                      <input
                                          value={state.draft_due.clone()}
                                          placeholder="e.g. tomorrow, 2028, march, wed, 3:23pm, 2026-02-20"
                                          oninput={{
                                              let modal_state = modal_state.clone();
                                              Callback::from(move |e: web_sys::InputEvent| {
                                                  let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                  if let Some(mut current) = (*modal_state).clone() {
                                                      current.draft_due = input.value();
                                                      current.error = None;
                                                      modal_state.set(Some(current));
                                                  }
                                              })
                                          }}
                                      />
                                  </div>
                                  {
                                      if state.allow_recurrence {
                                          html! {
                                              <>
                                                  <div class="field">
                                                      <label>{ "Recurrence" }</label>
                                                      <select
                                                          class="tag-select"
                                                          value={state.recurrence_pattern.clone()}
                                                          onchange={on_recurrence_pattern_change}
                                                      >
                                                          <option value="none">{ "None" }</option>
                                                          <option value="daily">{ "Daily" }</option>
                                                          <option value="weekly">{ "Weekly" }</option>
                                                          <option value="months">{ "Months" }</option>
                                                          <option value="monthly">{ "Monthly" }</option>
                                                          <option value="yearly">{ "Yearly" }</option>
                                                      </select>
                                                  </div>
                                                  {
                                                      if state.recurrence_pattern != "none" {
                                                          html! {
                                                              <div class="field">
                                                                  <label>{ "Recurring Time" }</label>
                                                                  <input
                                                                      value={state.recurrence_time.clone()}
                                                                      placeholder="e.g. 03:23pm or 15:23"
                                                                      oninput={on_recurrence_time_change}
                                                                  />
                                                              </div>
                                                          }
                                                      } else {
                                                          html! {}
                                                      }
                                                  }
                                                  {
                                                      if state.recurrence_pattern == "weekly" {
                                                          html! {
                                                              <div class="field">
                                                                  <label>{ "Weekly Days" }</label>
                                                                  <div class="toggle-grid">
                                                                      {
                                                                          for WEEKDAY_KEYS.iter().map(|day| {
                                                                              let day_key = (*day).to_string();
                                                                              let day_label = day_key.to_ascii_uppercase();
                                                                              let is_active = state.recurrence_days.iter().any(|entry| entry == &day_key);
                                                                              let modal_state = modal_state.clone();
                                                                              html! {
                                                                                  <button
                                                                                      type="button"
                                                                                      class={classes!("toggle-btn", is_active.then_some("active"))}
                                                                                      onclick={Callback::from(move |_| {
                                                                                          if let Some(mut current) = (*modal_state).clone() {
                                                                                              if current.recurrence_days.iter().any(|entry| entry == &day_key) {
                                                                                                  current.recurrence_days.retain(|entry| entry != &day_key);
                                                                                              } else {
                                                                                                  current.recurrence_days.push(day_key.clone());
                                                                                              }
                                                                                              current.error = None;
                                                                                              modal_state.set(Some(current));
                                                                                          }
                                                                                      })}
                                                                                  >
                                                                                      { day_label }
                                                                                  </button>
                                                                              }
                                                                          })
                                                                      }
                                                                  </div>
                                                              </div>
                                                          }
                                                      } else {
                                                          html! {}
                                                      }
                                                  }
                                                  {
                                                      if state.recurrence_pattern == "monthly"
                                                          || state.recurrence_pattern == "months"
                                                          || state.recurrence_pattern == "yearly"
                                                      {
                                                          html! {
                                                              <>
                                                                  <div class="field">
                                                                      <label>{ "Months" }</label>
                                                                      <div class="toggle-grid months">
                                                                          {
                                                                              for MONTH_KEYS.iter().map(|month| {
                                                                                  let month_key = (*month).to_string();
                                                                                  let month_label = month_key.to_ascii_uppercase();
                                                                                  let is_active = state.recurrence_months.iter().any(|entry| entry == &month_key);
                                                                                  let modal_state = modal_state.clone();
                                                                                  html! {
                                                                                      <button
                                                                                          type="button"
                                                                                          class={classes!("toggle-btn", is_active.then_some("active"))}
                                                                                          onclick={Callback::from(move |_| {
                                                                                              if let Some(mut current) = (*modal_state).clone() {
                                                                                                  if current.recurrence_months.iter().any(|entry| entry == &month_key) {
                                                                                                      current.recurrence_months.retain(|entry| entry != &month_key);
                                                                                                  } else {
                                                                                                      current.recurrence_months.push(month_key.clone());
                                                                                                  }
                                                                                                  current.error = None;
                                                                                                  modal_state.set(Some(current));
                                                                                              }
                                                                                          })}
                                                                                      >
                                                                                          { month_label }
                                                                                      </button>
                                                                                  }
                                                                              })
                                                                          }
                                                                      </div>
                                                                  </div>
                                                                  <div class="field">
                                                                      <label>{ "Month Day(s)" }</label>
                                                                      <input
                                                                          value={state.recurrence_month_day.clone()}
                                                                          placeholder="e.g. 1 or 1,15,28"
                                                                          oninput={on_recurrence_month_day_change}
                                                                      />
                                                                  </div>
                                                              </>
                                                          }
                                                      } else {
                                                          html! {}
                                                      }
                                                  }
                                              </>
                                          }
                                      } else {
                                          html! {
                                              <div class="field">
                                                  <label>{ "Recurrence" }</label>
                                                  <div class="field-help">
                                                      { "Recurrence is disabled for tasks managed by imported calendars." }
                                                  </div>
                                              </div>
                                          }
                                      }
                                  }
                                  <div class="footer">
                                      <button
                                          id="modal-cancel-btn"
                                          type="button"
                                          class="btn"
                                          onclick={on_modal_close_click.clone()}
                                      >
                                          { "Cancel" }
                                      </button>
                                      <button
                                          id="modal-save-btn"
                                          type="button"
                                          class="btn"
                                          onclick={on_save_click}
                                          disabled={is_busy}
                                      >
                                          { if is_busy { "Saving..." } else { "Save" } }
                                      </button>
                                  </div>
                              </div>
                          </div>
                      </div>
                  }
              } else {
                  html! {}
              }

}
