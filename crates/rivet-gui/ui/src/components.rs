use std::collections::BTreeSet;

use rivet_gui_shared::{TaskDto, TaskStatus};
use uuid::Uuid;
use yew::{Callback, Html, Properties, function_component, html};

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub active: String,
    pub on_nav: Callback<String>,
}

#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let make_item = |key: &str, label: &str| {
        let active = props.active == key;
        let class = if active { "item active" } else { "item" };
        let on_nav = props.on_nav.clone();
        let key_string = key.to_string();
        html! {
            <div class={class} onclick={move |_| on_nav.emit(key_string.clone())}>
                { label }
            </div>
        }
    };

    html! {
        <div class="panel sidebar">
            <div class="header">{ "Views" }</div>
            { make_item("inbox", "Inbox") }
            { make_item("all", "Tasks") }
            { make_item("projects", "Projects") }
            { make_item("tags", "Tags") }
            { make_item("settings", "Settings") }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TaskListProps {
    pub tasks: Vec<TaskDto>,
    pub selected: Option<Uuid>,
    pub selected_ids: BTreeSet<Uuid>,
    pub on_select: Callback<Uuid>,
    pub on_toggle_select: Callback<Uuid>,
}

#[function_component(TaskList)]
pub fn task_list(props: &TaskListProps) -> Html {
    html! {
        <div class="panel list">
            <div class="header">{ "Tasks" }</div>
            {
                for props.tasks.iter().map(|task| {
                    let id = task.uuid;
                    let selected = props.selected == Some(id);
                    let class = if selected { "row selected" } else { "row" };
                    let on_select = props.on_select.clone();
                    let on_toggle_select = props.on_toggle_select.clone();
                    let checked = props.selected_ids.contains(&id);

                    let dot_class = match task.status {
                        TaskStatus::Pending => "dot pending",
                        TaskStatus::Completed => "dot done",
                        TaskStatus::Deleted => "dot deleted",
                        TaskStatus::Waiting => "dot waiting",
                    };

                    let meta_project = task.project.clone().unwrap_or_else(|| "—".to_string());
                    let due = task.due.clone().unwrap_or_default();

                    html! {
                        <div class={class} onclick={move |_| on_select.emit(id)}>
                            <button
                                class={if checked { "selector on" } else { "selector" }}
                                onclick={move |e: yew::MouseEvent| {
                                    e.stop_propagation();
                                    on_toggle_select.emit(id);
                                }}
                            >
                                { if checked { "✓" } else { "" } }
                            </button>
                            <div class={dot_class}></div>
                            <div>
                                <div>{ &task.description }</div>
                                <div style="margin-top:4px;display:flex;gap:6px;flex-wrap:wrap;">
                                    <span class="badge">{ format!("project:{meta_project}") }</span>
                                    {
                                        for task.tags.iter().take(4).map(|tag| html! {
                                            <span class="badge">{ format!("#{tag}") }</span>
                                        })
                                    }
                                </div>
                            </div>
                            <div>
                                {
                                    if due.is_empty() {
                                        html! {}
                                    } else {
                                        html! { <span class="badge">{ format!("due:{due}") }</span> }
                                    }
                                }
                            </div>
                        </div>
                    }
                })
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DetailsProps {
    pub task: Option<TaskDto>,
    pub on_done: Callback<Uuid>,
    pub on_delete: Callback<Uuid>,
    pub on_edit: Callback<TaskDto>,
}

#[function_component(Details)]
pub fn details(props: &DetailsProps) -> Html {
    let Some(task) = &props.task else {
        return html! {
            <div class="panel">
                <div class="header">{ "Details" }</div>
                <div class="details">
                    <div style="color:var(--muted);">{ "Select a task to inspect and edit details." }</div>
                </div>
            </div>
        };
    };

    let on_done = props.on_done.clone();
    let on_delete = props.on_delete.clone();
    let on_edit = props.on_edit.clone();
    let id = task.uuid;
    let task_for_edit = task.clone();
    let can_mark_done = matches!(task.status, TaskStatus::Pending | TaskStatus::Waiting);

    html! {
        <div class="panel">
            <div class="header">{ "Details" }</div>
            <div class="details">
                <div style="font-family:var(--mono);color:var(--muted);">{ format!("uuid: {id}") }</div>
                <div style="font-size:1.15rem;font-weight:700;">{ &task.description }</div>

                <div class="kv">
                    <strong>{ "project" }</strong>
                    <div>{ task.project.clone().unwrap_or_else(|| "—".to_string()) }</div>
                </div>

                <div class="kv">
                    <strong>{ "tags" }</strong>
                    <div>
                        {
                            if task.tags.is_empty() {
                                html! { <span style="color:var(--muted);">{ "—" }</span> }
                            } else {
                                html! {
                                    <>
                                        { for task.tags.iter().map(|tag| html!{ <span class="badge" style="margin-right:6px;">{ format!("#{tag}") }</span> }) }
                                    </>
                                }
                            }
                        }
                    </div>
                </div>

                <div class="kv">
                    <strong>{ "due" }</strong>
                    <div>{ task.due.clone().unwrap_or_else(|| "—".to_string()) }</div>
                </div>

                <div class="kv">
                    <strong>{ "status" }</strong>
                    <div>{ format!("{:?}", task.status) }</div>
                </div>

                <div class="actions">
                    <button class="btn" onclick={move |_| on_edit.emit(task_for_edit.clone())}>{ "Edit" }</button>
                    {
                        if can_mark_done {
                            html! { <button class="btn ok" onclick={move |_| on_done.emit(id)}>{ "Done" }</button> }
                        } else {
                            html! {}
                        }
                    }
                    <button class="btn danger" onclick={move |_| on_delete.emit(id)}>{ "Delete" }</button>
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct FacetPanelProps {
    pub title: String,
    pub items: Vec<(String, usize)>,
    pub selected: Option<String>,
    pub on_select: Callback<Option<String>>,
}

#[function_component(FacetPanel)]
pub fn facet_panel(props: &FacetPanelProps) -> Html {
    let on_select_all = props.on_select.clone();

    html! {
        <div class="panel">
            <div class="header">{ &props.title }</div>
            <div class="details">
                <div
                    class={if props.selected.is_none() { "facet active" } else { "facet" }}
                    onclick={move |_| on_select_all.emit(None)}
                >
                    <span>{ "All" }</span>
                </div>

                {
                    for props.items.iter().map(|(item, count)| {
                        let item_name = item.clone();
                        let on_select = props.on_select.clone();
                        let is_active = props.selected.as_deref() == Some(item.as_str());
                        let class = if is_active { "facet active" } else { "facet" };
                        html! {
                            <div class={class} onclick={move |_| on_select.emit(Some(item_name.clone()))}>
                                <span>{ item }</span>
                                <span class="badge">{ *count }</span>
                            </div>
                        }
                    })
                }
            </div>
        </div>
    }
}
