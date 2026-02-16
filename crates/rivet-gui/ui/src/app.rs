use std::collections::{BTreeMap, BTreeSet};

use gloo::console::log;
use rivet_gui_shared::{
    TaskCreate, TaskDto, TaskIdArg, TaskPatch, TaskStatus, TaskUpdateArgs, TasksListArgs,
};
use serde::Serialize;
use uuid::Uuid;
use yew::{
    Callback, Html, TargetCast, classes, function_component, html, use_effect_with, use_state,
};

use crate::api::invoke_tauri;
use crate::components::{Details, FacetPanel, Sidebar, TaskList};

#[derive(Clone, PartialEq)]
struct ModalState {
    mode: ModalMode,
    draft_desc: String,
    draft_project: String,
    draft_tags: String,
    draft_due: String,
    error: Option<String>,
}

#[derive(Clone, PartialEq)]
enum ModalMode {
    Add,
    Edit(Uuid),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    Day,
    Night,
}

impl ThemeMode {
    fn as_class(self) -> &'static str {
        match self {
            Self::Day => "theme-day",
            Self::Night => "theme-night",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Day => Self::Night,
            Self::Night => Self::Day,
        }
    }

    fn storage_value(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Night => "night",
        }
    }

    fn toggle_label(self) -> &'static str {
        match self {
            Self::Day => "Night",
            Self::Night => "Day",
        }
    }
}

const THEME_STORAGE_KEY: &str = "rivet.theme";

#[function_component(App)]
pub fn app() -> Html {
    let theme = use_state(load_theme_mode);
    let active_view = use_state(|| "inbox".to_string());
    let search = use_state(String::new);
    let refresh_tick = use_state(|| 0_u64);

    let tasks = use_state(Vec::<TaskDto>::new);
    let selected = use_state(|| None::<Uuid>);
    let bulk_selected = use_state(BTreeSet::<Uuid>::new);
    let active_project = use_state(|| None::<String>);
    let active_tag = use_state(|| None::<String>);
    let modal_state = use_state(|| None::<ModalState>);

    {
        use_effect_with((), move |_| {
            ui_debug("app.mounted", "frontend mounted and hooks initialized");
            || ()
        });
    }

    {
        let active_view = active_view.clone();
        let refresh_tick = refresh_tick.clone();
        let tasks = tasks.clone();

        use_effect_with(
            ((*active_view).clone(), *refresh_tick),
            move |(view, tick)| {
                let tasks = tasks.clone();
                let view = view.clone();
                let tick = *tick;

                wasm_bindgen_futures::spawn_local(async move {
                    tracing::info!(view = %view, tick, "refreshing task list");

                    let status = if view == "completed" {
                        TaskStatus::Completed
                    } else {
                        TaskStatus::Pending
                    };

                    let args = TasksListArgs {
                        query: None,
                        status: Some(status),
                        project: None,
                        tag: None,
                    };

                    match invoke_tauri::<Vec<TaskDto>, _>("tasks_list", &args).await {
                        Ok(list) => tasks.set(list),
                        Err(err) => tracing::error!(error = %err, "tasks_list failed"),
                    }
                });

                || ()
            },
        );
    }

    let visible_tasks = {
        let query = (*search).clone();
        filter_visible_tasks(
            &tasks,
            &active_view,
            &query,
            active_project.as_deref(),
            active_tag.as_deref(),
        )
    };

    let selected_task =
        (*selected).and_then(|id| visible_tasks.iter().find(|task| task.uuid == id).cloned());

    let project_facets = build_project_facets(&tasks);
    let tag_facets = build_tag_facets(&tasks);

    let on_nav = {
        let active_view = active_view.clone();
        let selected = selected.clone();
        let bulk_selected = bulk_selected.clone();
        let active_project = active_project.clone();
        let active_tag = active_tag.clone();
        Callback::from(move |view: String| {
            active_view.set(view);
            selected.set(None);
            bulk_selected.set(BTreeSet::new());
            active_project.set(None);
            active_tag.set(None);
        })
    };

    let on_select = {
        let selected = selected.clone();
        Callback::from(move |id: Uuid| selected.set(Some(id)))
    };

    let on_toggle_select = {
        let bulk_selected = bulk_selected.clone();
        Callback::from(move |id: Uuid| {
            let mut next = (*bulk_selected).clone();
            if next.contains(&id) {
                next.remove(&id);
            } else {
                next.insert(id);
            }
            bulk_selected.set(next);
        })
    };

    let on_choose_project = {
        let active_project = active_project.clone();
        let selected = selected.clone();
        let bulk_selected = bulk_selected.clone();
        Callback::from(move |project: Option<String>| {
            active_project.set(project);
            selected.set(None);
            bulk_selected.set(BTreeSet::new());
        })
    };

    let on_choose_tag = {
        let active_tag = active_tag.clone();
        let selected = selected.clone();
        let bulk_selected = bulk_selected.clone();
        Callback::from(move |tag: Option<String>| {
            active_tag.set(tag);
            selected.set(None);
            bulk_selected.set(BTreeSet::new());
        })
    };

    let on_add_click = {
        let modal_state = modal_state.clone();
        Callback::from(move |_| {
            modal_state.set(Some(ModalState {
                mode: ModalMode::Add,
                draft_desc: String::new(),
                draft_project: String::new(),
                draft_tags: String::new(),
                draft_due: String::new(),
                error: None,
            }));
            ui_debug("action.add_modal.open", "clicked Add Task");
        })
    };

    let on_toggle_theme = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let next = (*theme).next();
            save_theme_mode(next);
            theme.set(next);
        })
    };

    let on_done = {
        let refresh_tick = refresh_tick.clone();
        let selected = selected.clone();
        let bulk_selected = bulk_selected.clone();
        Callback::from(move |uuid: Uuid| {
            let refresh_tick = refresh_tick.clone();
            let selected = selected.clone();
            let bulk_selected = bulk_selected.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let arg = TaskIdArg { uuid };
                match invoke_tauri::<TaskDto, _>("task_done", &arg).await {
                    Ok(_) => {
                        selected.set(None);
                        bulk_selected.set(BTreeSet::new());
                        refresh_tick.set((*refresh_tick).saturating_add(1));
                    }
                    Err(err) => tracing::error!(error = %err, "task_done failed"),
                }
            });
        })
    };

    let on_delete = {
        let refresh_tick = refresh_tick.clone();
        let selected = selected.clone();
        let bulk_selected = bulk_selected.clone();
        Callback::from(move |uuid: Uuid| {
            let refresh_tick = refresh_tick.clone();
            let selected = selected.clone();
            let bulk_selected = bulk_selected.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let arg = TaskIdArg { uuid };
                match invoke_tauri::<(), _>("task_delete", &arg).await {
                    Ok(()) => {
                        selected.set(None);
                        bulk_selected.set(BTreeSet::new());
                        refresh_tick.set((*refresh_tick).saturating_add(1));
                    }
                    Err(err) => tracing::error!(error = %err, "task_delete failed"),
                }
            });
        })
    };

    let on_bulk_done = {
        let bulk_selected = bulk_selected.clone();
        let refresh_tick = refresh_tick.clone();
        let selected = selected.clone();
        Callback::from(move |_| {
            let ids: Vec<Uuid> = (*bulk_selected).iter().copied().collect();
            if ids.is_empty() {
                return;
            }

            let bulk_selected = bulk_selected.clone();
            let refresh_tick = refresh_tick.clone();
            let selected = selected.clone();

            wasm_bindgen_futures::spawn_local(async move {
                for uuid in ids {
                    let arg = TaskIdArg { uuid };
                    if let Err(err) = invoke_tauri::<TaskDto, _>("task_done", &arg).await {
                        tracing::error!(error = %err, %uuid, "bulk task_done failed");
                    }
                }

                selected.set(None);
                bulk_selected.set(BTreeSet::new());
                refresh_tick.set((*refresh_tick).saturating_add(1));
            });
        })
    };

    let on_bulk_delete = {
        let bulk_selected = bulk_selected.clone();
        let refresh_tick = refresh_tick.clone();
        let selected = selected.clone();
        Callback::from(move |_| {
            let ids: Vec<Uuid> = (*bulk_selected).iter().copied().collect();
            if ids.is_empty() {
                return;
            }

            let bulk_selected = bulk_selected.clone();
            let refresh_tick = refresh_tick.clone();
            let selected = selected.clone();

            wasm_bindgen_futures::spawn_local(async move {
                for uuid in ids {
                    let arg = TaskIdArg { uuid };
                    if let Err(err) = invoke_tauri::<(), _>("task_delete", &arg).await {
                        tracing::error!(error = %err, %uuid, "bulk task_delete failed");
                    }
                }

                selected.set(None);
                bulk_selected.set(BTreeSet::new());
                refresh_tick.set((*refresh_tick).saturating_add(1));
            });
        })
    };

    let on_edit = {
        let modal_state = modal_state.clone();
        Callback::from(move |task: TaskDto| {
            modal_state.set(Some(ModalState {
                mode: ModalMode::Edit(task.uuid),
                draft_desc: task.description,
                draft_project: task.project.unwrap_or_default(),
                draft_tags: task.tags.join(" "),
                draft_due: task.due.unwrap_or_default(),
                error: None,
            }));
        })
    };

    let close_modal = {
        let modal_state = modal_state.clone();
        Callback::from(move |_| {
            modal_state.set(None);
            ui_debug("action.modal.cancel", "Cancel clicked, closing modal");
        })
    };

    let on_modal_close_click = {
        let close_modal = close_modal.clone();
        Callback::from(move |_| close_modal.emit(()))
    };

    let on_modal_submit = {
        let modal_state = modal_state.clone();
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |state: ModalState| {
            ui_debug(
                "action.modal.submit",
                &format!(
                    "mode={}, desc_len={}",
                    match state.mode {
                        ModalMode::Add => "add",
                        ModalMode::Edit(_) => "edit",
                    },
                    state.draft_desc.len()
                ),
            );
            ui_backend_log(
                "action.modal.submit",
                &format!(
                    "mode={}, desc_len={}",
                    match state.mode {
                        ModalMode::Add => "add",
                        ModalMode::Edit(_) => "edit",
                    },
                    state.draft_desc.len()
                ),
            );
            let modal_state = modal_state.clone();
            let refresh_tick = refresh_tick.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if state.draft_desc.trim().is_empty() {
                    let mut next = state.clone();
                    next.error = Some("Description is required.".to_string());
                    modal_state.set(Some(next));
                    return;
                }

                match state.mode {
                    ModalMode::Add => {
                        let create = TaskCreate {
                            description: state.draft_desc.trim().to_string(),
                            project: optional_text(&state.draft_project),
                            tags: split_tags(&state.draft_tags),
                            priority: None,
                            due: optional_text(&state.draft_due),
                            wait: None,
                            scheduled: None,
                        };

                        ui_debug("invoke.task_add.begin", "calling tauri command task_add");
                        ui_backend_log("invoke.task_add.begin", "calling tauri command task_add");
                        if let Err(err) = invoke_tauri::<TaskDto, _>("task_add", &create).await {
                            tracing::error!(error = %err, "task_add failed");
                            ui_debug("invoke.task_add.error", &err);
                            ui_backend_log("invoke.task_add.error", &err);
                            let mut next = state.clone();
                            next.error = Some(format!("Save failed: {err}"));
                            modal_state.set(Some(next));
                            return;
                        }
                        ui_debug("invoke.task_add.ok", "task_add succeeded");
                        ui_backend_log("invoke.task_add.ok", "task_add succeeded");
                    }
                    ModalMode::Edit(uuid) => {
                        let update = TaskUpdateArgs {
                            uuid,
                            patch: TaskPatch {
                                description: Some(state.draft_desc.trim().to_string()),
                                project: Some(optional_text(&state.draft_project)),
                                tags: Some(split_tags(&state.draft_tags)),
                                due: Some(optional_text(&state.draft_due)),
                                ..TaskPatch::default()
                            },
                        };

                        ui_debug(
                            "invoke.task_update.begin",
                            &format!("calling tauri command task_update uuid={uuid}"),
                        );
                        ui_backend_log(
                            "invoke.task_update.begin",
                            &format!("calling tauri command task_update uuid={uuid}"),
                        );
                        if let Err(err) = invoke_tauri::<TaskDto, _>("task_update", &update).await {
                            tracing::error!(error = %err, "task_update failed");
                            ui_debug("invoke.task_update.error", &err);
                            ui_backend_log("invoke.task_update.error", &err);
                            let mut next = state.clone();
                            next.error = Some(format!("Save failed: {err}"));
                            modal_state.set(Some(next));
                            return;
                        }
                        ui_debug("invoke.task_update.ok", "task_update succeeded");
                        ui_backend_log("invoke.task_update.ok", "task_update succeeded");
                    }
                }

                ui_debug("action.modal.close", "save complete, closing modal");
                ui_backend_log("action.modal.close", "save complete, closing modal");
                modal_state.set(None);
                refresh_tick.set((*refresh_tick).saturating_add(1));
            });
        })
    };

    let bulk_count = (*bulk_selected).len();

    html! {
        <div class={classes!("app", (*theme).as_class())}>
            <div class="topbar">
                <div class="brand">{ "Rivet" }</div>
                <div class="search">
                    <input
                        value={(*search).clone()}
                        placeholder="Search tasks"
                        oninput={{
                            let search = search.clone();
                            Callback::from(move |e: web_sys::InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                search.set(input.value());
                            })
                        }}
                    />
                </div>
                {
                    if bulk_count > 0 {
                        html! {
                            <>
                                <button class="btn ok" onclick={on_bulk_done.clone()}>{ format!("Done {bulk_count}") }</button>
                                <button class="btn danger" onclick={on_bulk_delete.clone()}>{ format!("Delete {bulk_count}") }</button>
                            </>
                        }
                    } else {
                        html! {}
                    }
                }
                <button class="btn" onclick={on_add_click}>{ "Add Task" }</button>
                <button class="btn" onclick={on_toggle_theme}>{ (*theme).toggle_label() }</button>
            </div>

            <div class="main">
                <Sidebar active={(*active_view).clone()} on_nav={on_nav} />

                {
                    if *active_view == "settings" {
                        html! {
                            <>
                                <div class="panel list">
                                    <div class="header">{ "Settings" }</div>
                                    <div class="details">
                                        <div>{ "The desktop UI is a thin client over the core Rivet datastore." }</div>
                                        <div class="kv"><strong>{ "view" }</strong><div>{ "settings" }</div></div>
                                        <div class="kv"><strong>{ "status" }</strong><div>{ "core + tauri bridge active" }</div></div>
                                        <div class="kv"><strong>{ "workflow" }</strong><div>{ "Use context/report commands in CLI for advanced behavior." }</div></div>
                                    </div>
                                </div>
                                <div class="panel">
                                    <div class="header">{ "Current Data" }</div>
                                    <div class="details">
                                        <div class="kv"><strong>{ "tasks loaded" }</strong><div>{ tasks.len() }</div></div>
                                        <div class="kv"><strong>{ "selected" }</strong><div>{ bulk_count }</div></div>
                                    </div>
                                </div>
                            </>
                        }
                    } else {
                        html! {
                            <>
                                <TaskList
                                    tasks={visible_tasks.clone()}
                                    selected={*selected}
                                    selected_ids={(*bulk_selected).clone()}
                                    on_select={on_select}
                                    on_toggle_select={on_toggle_select}
                                />
                                {
                                    if *active_view == "projects" && selected_task.is_none() {
                                        html! {
                                            <FacetPanel
                                                title={"Projects".to_string()}
                                                selected={(*active_project).clone()}
                                                items={project_facets}
                                                on_select={on_choose_project}
                                            />
                                        }
                                    } else if *active_view == "tags" && selected_task.is_none() {
                                        html! {
                                            <FacetPanel
                                                title={"Tags".to_string()}
                                                selected={(*active_tag).clone()}
                                                items={tag_facets}
                                                on_select={on_choose_tag}
                                            />
                                        }
                                    } else {
                                        html! {
                                            <Details task={selected_task} on_done={on_done} on_delete={on_delete} on_edit={on_edit} />
                                        }
                                    }
                                }
                            </>
                        }
                    }
                }
            </div>

            {
                if let Some(state) = (*modal_state).clone() {
                    let submit_state = state.clone();
                    let on_submit_form = {
                        let on_modal_submit = on_modal_submit.clone();
                        let submit_state = submit_state.clone();
                        Callback::from(move |e: web_sys::SubmitEvent| {
                            e.prevent_default();
                            ui_debug("form.submit.handler", "submit event fired");
                            on_modal_submit.emit(submit_state.clone());
                        })
                    };
                    let on_save_mousedown = {
                        let on_modal_submit = on_modal_submit.clone();
                        let submit_state = submit_state.clone();
                        Callback::from(move |e: web_sys::MouseEvent| {
                            e.prevent_default();
                            ui_debug("button.save.mousedown", "save mousedown fired");
                            ui_backend_log("button.save.mousedown", "save mousedown fired");
                            on_modal_submit.emit(submit_state.clone());
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
                                <form class="content" onsubmit={on_submit_form}>
                                    {
                                        if let Some(err) = state.error.clone() {
                                            html! { <div class="form-error">{ err }</div> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    <div class="field">
                                        <label>{ "Description" }</label>
                                        <input
                                            value={state.draft_desc.clone()}
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
                                        <label>{ "Tags (space separated)" }</label>
                                        <input
                                            value={state.draft_tags.clone()}
                                            oninput={{
                                                let modal_state = modal_state.clone();
                                                Callback::from(move |e: web_sys::InputEvent| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    if let Some(mut current) = (*modal_state).clone() {
                                                        current.draft_tags = input.value();
                                                        current.error = None;
                                                        modal_state.set(Some(current));
                                                    }
                                                })
                                            }}
                                        />
                                    </div>
                                    <div class="field">
                                        <label>{ "Due" }</label>
                                        <input
                                            value={state.draft_due.clone()}
                                            placeholder="e.g. tomorrow or 2026-02-20"
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
                                            onmousedown={on_save_mousedown}
                                        >
                                            { "Save" }
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

fn load_theme_mode() -> ThemeMode {
    let stored = web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(THEME_STORAGE_KEY).ok().flatten());

    match stored.as_deref() {
        Some("night") => ThemeMode::Night,
        _ => ThemeMode::Day,
    }
}

fn save_theme_mode(theme: ThemeMode) {
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        let _ = storage.set_item(THEME_STORAGE_KEY, theme.storage_value());
    }
}

fn optional_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn split_tags(text: &str) -> Vec<String> {
    text.split_whitespace().map(ToString::to_string).collect()
}

fn filter_visible_tasks(
    tasks: &[TaskDto],
    active_view: &str,
    query: &str,
    active_project: Option<&str>,
    active_tag: Option<&str>,
) -> Vec<TaskDto> {
    let q = query.to_ascii_lowercase();

    tasks
        .iter()
        .filter(|task| {
            if !q.is_empty() && !task.description.to_ascii_lowercase().contains(&q) {
                return false;
            }

            match active_view {
                "projects" => {
                    if let Some(project) = active_project {
                        task.project.as_deref() == Some(project)
                    } else {
                        true
                    }
                }
                "tags" => {
                    if let Some(tag) = active_tag {
                        task.tags.iter().any(|value| value == tag)
                    } else {
                        true
                    }
                }
                _ => true,
            }
        })
        .cloned()
        .collect()
}

fn build_project_facets(tasks: &[TaskDto]) -> Vec<(String, usize)> {
    let mut counts = BTreeMap::new();
    for task in tasks {
        if let Some(project) = task.project.as_ref() {
            *counts.entry(project.clone()).or_insert(0_usize) += 1;
        }
    }
    counts.into_iter().collect()
}

fn build_tag_facets(tasks: &[TaskDto]) -> Vec<(String, usize)> {
    let mut counts = BTreeMap::new();
    for task in tasks {
        for tag in &task.tags {
            *counts.entry(tag.clone()).or_insert(0_usize) += 1;
        }
    }
    counts.into_iter().collect()
}

fn ui_debug(event: &str, detail: &str) {
    tracing::debug!(event, detail, "ui-debug");
    log!(format!("[ui-debug] {event}: {detail}"));
}

fn ui_backend_log(event: &str, detail: &str) {
    let payload = UiLogPayload {
        event: event.to_string(),
        detail: detail.to_string(),
    };

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(err) = invoke_tauri::<(), _>("ui_log", &payload).await {
            log!(format!("[ui-backend-log] invoke failed: {err}"));
        }
    });
}

#[derive(Debug, Clone, Serialize)]
struct UiLogPayload {
    event: String,
    detail: String,
}
