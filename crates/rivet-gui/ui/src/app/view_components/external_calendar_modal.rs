#[derive(Properties, PartialEq)]
struct ExternalCalendarModalProps {
  modal_state:
    UseStateHandle<Option<ExternalCalendarModalState>>,
  busy:                 bool,
  on_close:
    Callback<MouseEvent>,
  on_submit:
    Callback<ExternalCalendarModalState>
}

#[function_component(ExternalCalendarModal)]
fn external_calendar_modal(
  props: &ExternalCalendarModalProps
) -> Html {
  let external_calendar_modal =
    props.modal_state.clone();
  if let Some(ext_modal) =
    (*external_calendar_modal).clone()
  {
    let submit_state = ext_modal.clone();
    let is_busy = props.busy;
    let on_save_click = {
      let on_submit_external_calendar =
        props.on_submit.clone();
      Callback::from(move |_| {
        on_submit_external_calendar
          .emit(submit_state.clone())
      })
    };

    html! {
        <div class="modal-backdrop" onclick={props.on_close.clone()}>
            <div class="modal modal-md" onclick={Callback::from(|e: yew::MouseEvent| e.stop_propagation())}>
                <div class="header">
                    {
                        match ext_modal.mode {
                            ExternalCalendarModalMode::Add => "Add External Calendar",
                            ExternalCalendarModalMode::Edit => "Edit External Calendar",
                        }
                    }
                </div>
                <div class="content">
                    {
                        if let Some(err) = ext_modal.error.clone() {
                            html! { <div class="form-error">{ err }</div> }
                        } else {
                            html! {}
                        }
                    }
                    <div class="field field-inline-check">
                        <label>{ "Enable This Calendar" }</label>
                        <input
                            type="checkbox"
                            checked={ext_modal.source.enabled}
                            disabled={ext_modal.source.imported_ics_file}
                            onchange={{
                                let external_calendar_modal = external_calendar_modal.clone();
                                Callback::from(move |e: web_sys::Event| {
                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                        if let Some(mut current) = (*external_calendar_modal).clone() {
                                            current.source.enabled = input.checked();
                                            current.error = None;
                                            external_calendar_modal.set(Some(current));
                                        }
                                    }
                                })
                            }}
                        />
                    </div>
                    {
                        if ext_modal.source.imported_ics_file {
                            html! {
                                <div class="field-help">
                                    { "Imported ICS calendars are local snapshots. Use Import ICS File again to refresh data." }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                    <div class="field">
                        <label>{ "Calendar Name" }</label>
                        <input
                            value={ext_modal.source.name.clone()}
                            oninput={{
                                let external_calendar_modal = external_calendar_modal.clone();
                                Callback::from(move |e: web_sys::InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    if let Some(mut current) = (*external_calendar_modal).clone() {
                                        current.source.name = input.value();
                                        current.error = None;
                                        external_calendar_modal.set(Some(current));
                                    }
                                })
                            }}
                        />
                    </div>
                    <div class="field">
                        <label>{ "Color" }</label>
                        <input
                            type="color"
                            value={ext_modal.source.color.clone()}
                            oninput={{
                                let external_calendar_modal = external_calendar_modal.clone();
                                Callback::from(move |e: web_sys::InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    if let Some(mut current) = (*external_calendar_modal).clone() {
                                        current.source.color = input.value();
                                        current.error = None;
                                        external_calendar_modal.set(Some(current));
                                    }
                                })
                            }}
                        />
                    </div>
                    <div class="field">
                        <label>{ "Location (ICS or webcal URL)" }</label>
                        <input
                            value={ext_modal.source.location.clone()}
                            placeholder="webcal://example.com/calendar.ics"
                            disabled={ext_modal.source.imported_ics_file}
                            oninput={{
                                let external_calendar_modal = external_calendar_modal.clone();
                                Callback::from(move |e: web_sys::InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    if let Some(mut current) = (*external_calendar_modal).clone() {
                                        current.source.location = input.value();
                                        current.error = None;
                                        external_calendar_modal.set(Some(current));
                                    }
                                })
                            }}
                        />
                    </div>
                    <div class="field">
                        <label>{ "Refresh Calendar" }</label>
                        <select
                            class="tag-select"
                            value={ext_modal.source.refresh_minutes.to_string()}
                            disabled={ext_modal.source.imported_ics_file}
                            onchange={{
                                let external_calendar_modal = external_calendar_modal.clone();
                                Callback::from(move |e: web_sys::Event| {
                                    let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                    if let Some(mut current) = (*external_calendar_modal).clone() {
                                        let parsed = select.value().parse::<u32>().ok().unwrap_or(30);
                                        current.source.refresh_minutes = parsed;
                                        current.error = None;
                                        external_calendar_modal.set(Some(current));
                                    }
                                })
                            }}
                        >
                            <option value="0">{ "Disabled (manual only)" }</option>
                            <option value="5">{ "Every 5 minutes" }</option>
                            <option value="15">{ "Every 15 minutes" }</option>
                            <option value="30">{ "Every 30 minutes" }</option>
                            <option value="60">{ "Every 60 minutes" }</option>
                            <option value="360">{ "Every 6 hours" }</option>
                            <option value="1440">{ "Every 24 hours" }</option>
                        </select>
                    </div>
                    <div class="field field-inline-check">
                        <label>{ "Read Only" }</label>
                        <input
                            type="checkbox"
                            checked={ext_modal.source.read_only}
                            onchange={{
                                let external_calendar_modal = external_calendar_modal.clone();
                                Callback::from(move |e: web_sys::Event| {
                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                        if let Some(mut current) = (*external_calendar_modal).clone() {
                                            current.source.read_only = input.checked();
                                            current.error = None;
                                            external_calendar_modal.set(Some(current));
                                        }
                                    }
                                })
                            }}
                        />
                    </div>
                    <div class="field field-inline-check">
                        <label>{ "Show Reminders" }</label>
                        <input
                            type="checkbox"
                            checked={ext_modal.source.show_reminders}
                            onchange={{
                                let external_calendar_modal = external_calendar_modal.clone();
                                Callback::from(move |e: web_sys::Event| {
                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                        if let Some(mut current) = (*external_calendar_modal).clone() {
                                            current.source.show_reminders = input.checked();
                                            current.error = None;
                                            external_calendar_modal.set(Some(current));
                                        }
                                    }
                                })
                            }}
                        />
                    </div>
                    <div class="field field-inline-check">
                        <label>{ "Offline Support" }</label>
                        <input
                            type="checkbox"
                            checked={ext_modal.source.offline_support}
                            onchange={{
                                let external_calendar_modal = external_calendar_modal.clone();
                                Callback::from(move |e: web_sys::Event| {
                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                        if let Some(mut current) = (*external_calendar_modal).clone() {
                                            current.source.offline_support = input.checked();
                                            current.error = None;
                                            external_calendar_modal.set(Some(current));
                                        }
                                    }
                                })
                            }}
                        />
                    </div>
                    <div class="footer">
                        <button type="button" class="btn" onclick={props.on_close.clone()}>{ "Cancel" }</button>
                        <button type="button" class="btn" onclick={on_save_click} disabled={is_busy}>
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
