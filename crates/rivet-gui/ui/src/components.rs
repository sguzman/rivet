use std::collections::{
    BTreeMap,
    BTreeSet,
};

use rivet_gui_shared::{TaskDto, TaskStatus};
use uuid::Uuid;
use web_sys::DragEvent;
use yew::{Callback, Html, Properties, classes, function_component, html};

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
    pub tag_colors: BTreeMap<String, String>,
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
                                            <span class="badge tag-badge" style={tag_badge_style(tag, &props.tag_colors)}>{ format!("#{tag}") }</span>
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
    pub tag_colors: BTreeMap<String, String>,
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
                                        {
                                            for task.tags.iter().map(|tag| html!{
                                                <span class="badge tag-badge" style={format!("margin-right:6px;{}", tag_badge_style(tag, &props.tag_colors))}>{ format!("#{tag}") }</span>
                                            })
                                        }
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

#[derive(Properties, PartialEq)]
pub struct KanbanBoardProps {
    pub tasks: Vec<TaskDto>,
    pub columns: Vec<String>,
    pub board_name: Option<String>,
    pub tag_colors: BTreeMap<String, String>,
    pub dragging_task: Option<Uuid>,
    pub drag_over_lane: Option<String>,
    pub on_move: Callback<(Uuid, String)>,
    pub on_drag_start: Callback<Uuid>,
    pub on_drag_end: Callback<()>,
    pub on_drag_over_lane: Callback<String>,
    pub on_edit: Callback<TaskDto>,
    pub on_done: Callback<Uuid>,
    pub on_delete: Callback<Uuid>,
}

#[function_component(KanbanBoard)]
pub fn kanban_board(props: &KanbanBoardProps) -> Html {
    let columns = if props.columns.is_empty() {
        vec![
            "todo".to_string(),
            "working".to_string(),
            "finished".to_string(),
        ]
    } else {
        props.columns.clone()
    };
    let default_lane = columns
        .first()
        .cloned()
        .unwrap_or_else(|| "todo".to_string());

    let board_label = props
        .board_name
        .clone()
        .unwrap_or_else(|| "No board selected".to_string());

    html! {
        <div class="panel kanban-panel">
            <div class="header">{ format!("Kanban: {board_label}") }</div>
            <div class="kanban-board">
                {
                    for columns.iter().enumerate().map(|(column_idx, column_key)| {
                        let column_key = column_key.clone();
                        let column_title = humanize_lane(&column_key);
                        let columns_for_filter = columns.clone();
                        let default_lane_for_filter = default_lane.clone();
                        let cards: Vec<TaskDto> = props
                            .tasks
                            .iter()
                            .filter(|task| {
                                kanban_lane_for_task(
                                    task,
                                    &columns_for_filter,
                                    &default_lane_for_filter,
                                ) == column_key
                            })
                            .cloned()
                            .collect();
                        let on_move = props.on_move.clone();
                        let on_drag_over_lane = props.on_drag_over_lane.clone();
                        let column_key_string = column_key.to_string();
                        let lane_for_dragover = column_key_string.clone();
                        let lane_for_dragenter = column_key_string.clone();
                        let is_drop_hint = props.drag_over_lane.as_deref() == Some(column_key.as_str());

                        let ondragover = Callback::from(move |event: DragEvent| {
                            event.prevent_default();
                            event.stop_propagation();
                            on_drag_over_lane.emit(lane_for_dragover.clone());
                        });

                        let on_drag_over_lane_enter = props.on_drag_over_lane.clone();
                        let ondragenter = Callback::from(move |event: DragEvent| {
                            event.prevent_default();
                            event.stop_propagation();
                            on_drag_over_lane_enter.emit(lane_for_dragenter.clone());
                        });

                        let on_drag_end = props.on_drag_end.clone();
                        let ondrop = Callback::from(move |event: DragEvent| {
                            event.prevent_default();
                            event.stop_propagation();
                            if let Some(data_transfer) = event.data_transfer() {
                                match data_transfer.get_data("text/plain") {
                                    Ok(raw_uuid) => {
                                        if let Ok(uuid) = Uuid::parse_str(raw_uuid.trim()) {
                                            on_move.emit((uuid, column_key_string.clone()));
                                        } else {
                                            tracing::warn!(raw_uuid, "failed to parse dragged task uuid");
                                        }
                                    }
                                    Err(error) => tracing::warn!(?error, "failed reading drag data"),
                                }
                            }
                            on_drag_end.emit(());
                        });

                        html! {
                            <div class={classes!("kanban-column", is_drop_hint.then_some("drop-hint"))} {ondragover} {ondragenter} {ondrop}>
                                <div class="kanban-column-header">
                                    <span>{ column_title.clone() }</span>
                                    <span class="badge">{ cards.len() }</span>
                                </div>
                                <div class="kanban-column-body">
                                    {
                                        if cards.is_empty() {
                                            html! { <div class="kanban-empty">{ "No tasks" }</div> }
                                        } else {
                                            html! {
                                                <>
                                                    {
                                                        for cards.into_iter().map(|task| {
                                                            let task_id = task.uuid;
                                                            let task_for_edit = task.clone();
                                                            let on_edit = props.on_edit.clone();
                                                            let on_done = props.on_done.clone();
                                                            let on_delete = props.on_delete.clone();
                                                            let on_move = props.on_move.clone();
                                                            let on_drag_start = props.on_drag_start.clone();
                                                            let on_drag_end = props.on_drag_end.clone();
                                                            let tag_colors = props.tag_colors.clone();
                                                            let is_dragging = props.dragging_task == Some(task_id);

                                                            let ondragstart = Callback::from(move |event: DragEvent| {
                                                                if let Some(data_transfer) = event.data_transfer() {
                                                                    let _ = data_transfer.set_data("text/plain", &task_id.to_string());
                                                                    data_transfer.set_drop_effect("move");
                                                                }
                                                                on_drag_start.emit(task_id);
                                                            });

                                                            let ondragend = Callback::from(move |_| {
                                                                on_drag_end.emit(());
                                                            });

                                                            let next_lane = {
                                                                if columns.is_empty() {
                                                                    column_key.clone()
                                                                } else {
                                                                    columns[(column_idx + 1) % columns.len()].clone()
                                                                }
                                                            };
                                                            let next_lane_label = format!(
                                                                "Move to {}",
                                                                humanize_lane(&next_lane)
                                                            );

                                                            html! {
                                                                <div class={classes!("kanban-card", is_dragging.then_some("dragging"))} draggable="true" {ondragstart} {ondragend}>
                                                                    <div class="kanban-card-title">{ &task.description }</div>
                                                                    <div class="kanban-card-meta">
                                                                        <span class="badge">
                                                                            {
                                                                                if let Some(project) = task.project.clone() {
                                                                                    format!("project:{project}")
                                                                                } else {
                                                                                    "project:—".to_string()
                                                                                }
                                                                            }
                                                                        </span>
                                                                        {
                                                                            if let Some(due) = task.due.clone() {
                                                                                html! { <span class="badge">{ format!("due:{due}") }</span> }
                                                                            } else {
                                                                                html! {}
                                                                            }
                                                                        }
                                                                    </div>
                                                                    <div class="kanban-card-meta">
                                                                        {
                                                                            for task.tags.iter().take(3).map(|tag| html! {
                                                                                <span class="badge tag-badge" style={tag_badge_style(tag, &tag_colors)}>{ format!("#{tag}") }</span>
                                                                            })
                                                                        }
                                                                    </div>
                                                                    <div class="kanban-card-actions">
                                                                        <button class="btn" onclick={move |_| on_edit.emit(task_for_edit.clone())}>{ "Edit" }</button>
                                                                        <button class="btn" onclick={move |_| on_move.emit((task_id, next_lane.clone()))}>{ next_lane_label }</button>
                                                                        {
                                                                            if matches!(task.status, TaskStatus::Pending | TaskStatus::Waiting) {
                                                                                html! { <button class="btn ok" onclick={move |_| on_done.emit(task_id)}>{ "Done" }</button> }
                                                                            } else {
                                                                                html! {}
                                                                            }
                                                                        }
                                                                        <button class="btn danger" onclick={move |_| on_delete.emit(task_id)}>{ "Delete" }</button>
                                                                    </div>
                                                                </div>
                                                            }
                                                        })
                                                    }
                                                </>
                                            }
                                        }
                                    }
                                </div>
                            </div>
                        }
                    })
                }
            </div>
        </div>
    }
}

fn kanban_lane_for_task(
    task: &TaskDto,
    columns: &[String],
    default_lane: &str,
) -> String {
    for tag in &task.tags {
        if let Some((key, value)) = tag.split_once(':')
            && key == "kanban"
        {
            if columns.iter().any(|column| column == value) {
                return value.to_string();
            }
            return default_lane.to_string();
        }
    }
    default_lane.to_string()
}

fn humanize_lane(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = first.to_ascii_uppercase().to_string();
                    out.push_str(&chars.as_str().to_ascii_lowercase());
                    out
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn tag_badge_style(
    tag: &str,
    tag_colors: &BTreeMap<String, String>,
) -> String {
    if let Some((key, _)) = tag.split_once(':')
        && let Some(color) = tag_colors.get(key)
    {
        return format!("--tag-key-color:{color};");
    }

    String::new()
}
