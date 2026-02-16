use rivet_gui_shared::{
    TaskCreate, TaskDto, TaskIdArg, TaskPatch, TaskStatus, TaskUpdateArgs, TasksListArgs,
};
use uuid::Uuid;
use yew::{Callback, Html, TargetCast, function_component, html, use_effect_with, use_state};

use crate::api::invoke_tauri;
use crate::components::{Details, Sidebar, TaskList};

#[derive(Clone, PartialEq)]
struct ModalState {
    mode: ModalMode,
    draft_desc: String,
    draft_project: String,
    draft_tags: String,
    draft_due: String,
}

#[derive(Clone, PartialEq)]
enum ModalMode {
    Add,
    Edit(Uuid),
}

#[function_component(App)]
pub fn app() -> Html {
    let active_view = use_state(|| "inbox".to_string());
    let search = use_state(String::new);

    let tasks = use_state(Vec::<TaskDto>::new);
    let selected = use_state(|| None::<Uuid>);
    let modal_state = use_state(|| None::<ModalState>);

    {
        let active_view = active_view.clone();
        let search = search.clone();
        let tasks = tasks.clone();

        use_effect_with(
            ((*active_view).clone(), (*search).clone()),
            move |(view, query)| {
                let tasks = tasks.clone();
                let view = view.clone();
                let query = query.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    tracing::info!(view = %view, query = %query, "refreshing task list");

                    let args = match view.as_str() {
                        "completed" => TasksListArgs {
                            query: if query.is_empty() { None } else { Some(query) },
                            status: Some(TaskStatus::Completed),
                            project: None,
                            tag: None,
                        },
                        _ => TasksListArgs {
                            query: if query.is_empty() { None } else { Some(query) },
                            status: Some(TaskStatus::Pending),
                            project: None,
                            tag: None,
                        },
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

    let selected_task =
        (*selected).and_then(|id| tasks.iter().find(|task| task.uuid == id).cloned());

    let on_nav = {
        let active_view = active_view.clone();
        let selected = selected.clone();
        Callback::from(move |view: String| {
            active_view.set(view);
            selected.set(None);
        })
    };

    let on_select = {
        let selected = selected.clone();
        Callback::from(move |id: Uuid| selected.set(Some(id)))
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
            }))
        })
    };

    let on_done = {
        let tasks = tasks.clone();
        Callback::from(move |uuid: Uuid| {
            let tasks = tasks.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let arg = TaskIdArg { uuid };
                match invoke_tauri::<TaskDto, _>("task_done", &arg).await {
                    Ok(updated) => {
                        let mut next = (*tasks).clone();
                        if let Some(task) = next.iter_mut().find(|task| task.uuid == uuid) {
                            *task = updated;
                        }
                        tasks.set(next);
                    }
                    Err(err) => tracing::error!(error = %err, "task_done failed"),
                }
            });
        })
    };

    let on_delete = {
        let tasks = tasks.clone();
        let selected = selected.clone();
        Callback::from(move |uuid: Uuid| {
            let tasks = tasks.clone();
            let selected = selected.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let arg = TaskIdArg { uuid };
                match invoke_tauri::<(), _>("task_delete", &arg).await {
                    Ok(()) => {
                        let mut next = (*tasks).clone();
                        next.retain(|task| task.uuid != uuid);
                        tasks.set(next);
                        selected.set(None);
                    }
                    Err(err) => tracing::error!(error = %err, "task_delete failed"),
                }
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
            }));
        })
    };

    let on_modal_close = {
        let modal_state = modal_state.clone();
        Callback::from(move |_| modal_state.set(None))
    };

    let on_modal_submit = {
        let modal_state = modal_state.clone();
        let tasks = tasks.clone();
        Callback::from(move |state: ModalState| {
            let tasks = tasks.clone();
            let modal_state = modal_state.clone();

            wasm_bindgen_futures::spawn_local(async move {
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

                        match invoke_tauri::<TaskDto, _>("task_add", &create).await {
                            Ok(task) => {
                                let mut next = (*tasks).clone();
                                next.push(task);
                                tasks.set(next);
                            }
                            Err(err) => tracing::error!(error = %err, "task_add failed"),
                        }
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

                        match invoke_tauri::<TaskDto, _>("task_update", &update).await {
                            Ok(updated) => {
                                let mut next = (*tasks).clone();
                                if let Some(task) = next.iter_mut().find(|task| task.uuid == uuid) {
                                    *task = updated;
                                }
                                tasks.set(next);
                            }
                            Err(err) => tracing::error!(error = %err, "task_update failed"),
                        }
                    }
                }

                modal_state.set(None);
            });
        })
    };

    html! {
        <div class="app">
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
                <button class="btn" onclick={on_add_click}>{ "Add Task" }</button>
            </div>

            <div class="main">
                <Sidebar active={(*active_view).clone()} on_nav={on_nav} />
                <TaskList tasks={(*tasks).clone()} selected={*selected} on_select={on_select} />
                <Details task={selected_task} on_done={on_done} on_delete={on_delete} on_edit={on_edit} />
            </div>

            {
                if let Some(state) = (*modal_state).clone() {
                    let state_for_submit = state.clone();
                    html! {
                        <div class="modal-backdrop" onclick={on_modal_close.clone()}>
                            <div class="modal" onclick={|e: web_sys::MouseEvent| e.stop_propagation()}>
                                <div class="header">
                                    {
                                        match state.mode {
                                            ModalMode::Add => "Add Task",
                                            ModalMode::Edit(_) => "Edit Task",
                                        }
                                    }
                                </div>
                                <div class="content">
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
                                                        modal_state.set(Some(current));
                                                    }
                                                })
                                            }}
                                        />
                                    </div>
                                </div>
                                <div class="footer">
                                    <button class="btn" onclick={on_modal_close.clone()}>{ "Cancel" }</button>
                                    <button class="btn" onclick={{
                                        let on_modal_submit = on_modal_submit.clone();
                                        Callback::from(move |_| on_modal_submit.emit(state_for_submit.clone()))
                                    }}>{ "Save" }</button>
                                </div>
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
