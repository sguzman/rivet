#[function_component(App)]
pub fn app() -> Html {
  let theme =
    use_state(load_theme_mode);
  let active_tab =
    use_state(load_workspace_tab);
  let tag_schema =
    use_state(load_tag_schema);
  let calendar_config =
    use_state(load_calendar_config);
  let calendar_view =
    use_state(load_calendar_view_mode);
  let calendar_focus_date = {
    let config_snapshot =
      (*calendar_config).clone();
    use_state(move || {
      today_in_timezone(
        resolve_calendar_timezone(
          &config_snapshot
        )
      )
    })
  };
  let active_view =
    use_state(|| "all".to_string());
  let kanban_boards =
    use_state(load_kanban_boards);
  let active_kanban_board = {
    let boards_snapshot =
      (*kanban_boards).clone();
    use_state(move || {
      load_active_kanban_board(
        &boards_snapshot
      )
    })
  };
  let dragging_kanban_task =
    use_state(|| None::<Uuid>);
  let drag_over_kanban_lane =
    use_state(|| None::<String>);
  let kanban_rename_open =
    use_state(|| false);
  let kanban_rename_input =
    use_state(String::new);
  let kanban_create_open =
    use_state(|| false);
  let kanban_create_input =
    use_state(String::new);
  let kanban_compact_cards =
    use_state(|| false);
  let external_calendars =
    use_state(load_external_calendars);
  let external_calendar_modal =
    use_state(|| {
      None::<ExternalCalendarModalState>
    });
  let external_calendar_delete_modal =
    use_state(|| {
      None::<ExternalCalendarDeleteState>
    });
  let external_calendar_busy =
    use_state(|| false);
  let external_calendar_last_sync =
    use_state(|| None::<String>);
  let due_notification_config =
    use_state(load_due_notification_config);
  let due_notification_sent =
    use_state(load_due_notification_sent);
  let due_notification_permission =
    use_state(
      browser_due_notification_permission
    );
  let search = use_state(String::new);
  let refresh_tick =
    use_state(|| 0_u64);
  let task_refresh_inflight =
    yew::functional::use_mut_ref(
      || false
    );
  let task_refresh_pending =
    yew::functional::use_mut_ref(
      || false
    );

  let tasks =
    use_state(Vec::<TaskDto>::new);
  let facet_tasks =
    use_state(Vec::<TaskDto>::new);
  let selected =
    use_state(|| None::<Uuid>);
  let bulk_selected =
    use_state(BTreeSet::<Uuid>::new);
  let active_project =
    use_state(|| None::<String>);
  let active_tag =
    use_state(|| None::<String>);
  let all_filter_project =
    use_state(|| None::<String>);
  let all_filter_tag =
    use_state(|| None::<String>);
  let all_filter_completion =
    use_state(|| "all".to_string());
  let all_filter_priority =
    use_state(|| "all".to_string());
  let all_filter_due =
    use_state(|| "all".to_string());
  let modal_state =
    use_state(|| None::<ModalState>);
  let modal_busy = use_state(|| false);
  let modal_submit_seq =
    use_state(|| 0_u64);

  {
    use_effect_with((), move |_| {
      ui_debug(
        "app.mounted",
        "frontend mounted and hooks \
         initialized"
      );
      || ()
    });
  }

  {
    let active_tab = active_tab.clone();
    use_effect_with(
      (*active_tab).clone(),
      move |tab| {
        save_workspace_tab(tab);
        tracing::debug!(
          tab = %tab,
          "persisted workspace tab"
        );
        || ()
      }
    );
  }

  {
    let calendar_view =
      calendar_view.clone();
    use_effect_with(
      *calendar_view,
      move |view| {
        save_calendar_view_mode(*view);
        tracing::debug!(
          view = %view.as_key(),
          "persisted calendar view mode"
        );
        || ()
      }
    );
  }

  {
    let external_calendars =
      external_calendars.clone();
    use_effect_with(
      (*external_calendars).clone(),
      move |calendars| {
        save_external_calendars(
          calendars
        );
        tracing::debug!(
          calendar_sources =
            calendars.len(),
          "persisted external \
           calendar sources"
        );
        || ()
      }
    );
  }

  {
    let due_notification_config =
      due_notification_config.clone();
    use_effect_with(
      (*due_notification_config).clone(),
      move |config| {
        save_due_notification_config(
          config
        );
        tracing::debug!(
          enabled = config.enabled,
          pre_notify_enabled = config
            .pre_notify_enabled,
          pre_notify_minutes = config
            .pre_notify_minutes,
          "persisted due notification \
           config"
        );
        || ()
      }
    );
  }

  {
    let due_notification_sent =
      due_notification_sent.clone();
    use_effect_with(
      (*due_notification_sent).clone(),
      move |sent| {
        save_due_notification_sent(sent);
        tracing::debug!(
          sent_keys = sent.len(),
          "persisted due notification \
           registry"
        );
        || ()
      }
    );
  }

  {
    let due_notification_permission =
      due_notification_permission
        .clone();
    use_effect_with((), move |_| {
      due_notification_permission.set(
        browser_due_notification_permission(),
      );
      || ()
    });
  }

  {
    let external_calendars =
      external_calendars.clone();
    let refresh_tick =
      refresh_tick.clone();
    let external_calendar_last_sync =
      external_calendar_last_sync
        .clone();
    use_effect_with(
      (*external_calendars).clone(),
      move |sources| {
        let mut intervals = Vec::new();

        for source in sources
          .iter()
          .cloned()
          .filter(|source| {
            source.enabled
              && !source
                .imported_ics_file
              && source.refresh_minutes
                > 0
          })
        {
          let refresh_tick =
            refresh_tick.clone();
          let external_calendar_last_sync =
            external_calendar_last_sync
              .clone();
          let period_ms = source
            .refresh_minutes
            .saturating_mul(60_000);

          intervals.push(Interval::new(
            period_ms,
            move || {
              let source =
                source.clone();
              let refresh_tick =
                refresh_tick.clone();
              let external_calendar_last_sync =
                external_calendar_last_sync
                  .clone();
              wasm_bindgen_futures::spawn_local(async move {
                match invoke_tauri::<
                  ExternalCalendarSyncResult,
                  _
                >(
                  "external_calendar_sync",
                  &source
                )
                .await
                {
                  | Ok(result) => {
                    tracing::info!(
                      calendar_id = %result.calendar_id,
                      created = result.created,
                      updated = result.updated,
                      deleted = result.deleted,
                      "external calendar auto sync succeeded"
                    );
                    external_calendar_last_sync
                      .set(Some(format!(
                        "Synced {}: +{} / ~{} / -{}",
                        source.name,
                        result.created,
                        result.updated,
                        result.deleted
                      )));
                    refresh_tick.set(
                      (*refresh_tick)
                        .saturating_add(1),
                    );
                  }
                  | Err(err) => {
                    tracing::error!(
                      calendar = %source.name,
                      error = %err,
                      "external calendar auto sync failed"
                    );
                    external_calendar_last_sync
                      .set(Some(format!(
                        "Sync failed for {}: {}",
                        source.name, err
                      )));
                  }
                }
              });
            },
          ));
        }

        move || drop(intervals)
      }
    );
  }

  {
    let kanban_boards =
      kanban_boards.clone();
    use_effect_with(
      (*kanban_boards).clone(),
      move |boards| {
        save_kanban_boards(boards);
        tracing::debug!(
          board_count = boards.len(),
          "persisted kanban boards"
        );
        || ()
      }
    );
  }

  {
    let active_kanban_board =
      active_kanban_board.clone();
    use_effect_with(
      (*active_kanban_board).clone(),
      move |active| {
        save_active_kanban_board(
          active.as_deref()
        );
        tracing::debug!(
          active_board = ?active,
          "persisted active kanban \
           board"
        );
        || ()
      }
    );
  }

  {
    let kanban_boards =
      kanban_boards.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    use_effect_with(
      (
        (*kanban_boards).clone(),
        (*active_kanban_board).clone()
      ),
      move |(boards, active)| {
        let contains_active = active
          .as_ref()
          .is_some_and(|id| {
            boards.iter().any(|board| {
              &board.id == id
            })
          });

        if !contains_active {
          let next =
            boards.first().map(
              |board| board.id.clone()
            );
          if next
            != *active_kanban_board
          {
            tracing::info!(
              active_board = ?active,
              next_board = ?next,
              "repairing active kanban \
               board selection"
            );
            active_kanban_board
              .set(next);
          }
        }

        || ()
      }
    );
  }

  {
    let active_tab = active_tab.clone();
    let active_view =
      active_view.clone();
    let tasks = tasks.clone();
    let facet_tasks =
      facet_tasks.clone();
    let task_refresh_inflight =
      task_refresh_inflight.clone();
    let task_refresh_pending =
      task_refresh_pending.clone();

    use_effect_with(
      *refresh_tick,
      move |tick| {
        let inflight = {
          let mut pending_state =
            task_refresh_inflight
              .borrow_mut();
          if *pending_state {
            true
          } else {
            *pending_state = true;
            false
          }
        };

        if inflight {
          *task_refresh_pending
            .borrow_mut() = true;
          tracing::debug!(
            tick = *tick,
            "coalescing task refresh \
             while prior fetch is in \
             flight"
          );
        } else {
          let tasks = tasks.clone();
          let facet_tasks =
            facet_tasks.clone();
          let active_tab =
            active_tab.clone();
          let active_view =
            active_view.clone();
          let task_refresh_inflight =
            task_refresh_inflight
              .clone();
          let task_refresh_pending =
            task_refresh_pending
              .clone();
          let mut current_tick =
            *tick;

          wasm_bindgen_futures::spawn_local(
            async move {
              loop {
                let tab =
                  (*active_tab).clone();
                let view =
                  (*active_view).clone();
                tracing::info!(
                  tab = %tab,
                  view = %view,
                  tick = current_tick,
                  "refreshing task list"
                );

                let status = if tab
                  == "kanban"
                  || tab == "calendar"
                  || view == "all"
                {
                  None
                } else {
                  Some(TaskStatus::Pending)
                };

                let args =
                  TasksListArgs {
                    query: None,
                    status,
                    project: None,
                    tag: None,
                  };

                match invoke_tauri::<
                  Vec<TaskDto>,
                  _
                >("tasks_list", &args)
                .await
                {
                  | Ok(list) => {
                    tasks.set(
                      list.clone()
                    );
                    facet_tasks
                      .set(list);
                  }
                  | Err(err) => tracing::error!(error = %err, "tasks_list failed")
                }

                let rerun = {
                  let mut pending =
                    task_refresh_pending
                      .borrow_mut();
                  let should_rerun =
                    *pending;
                  *pending = false;
                  should_rerun
                };

                if !rerun {
                  *task_refresh_inflight
                    .borrow_mut() =
                    false;
                  break;
                }

                current_tick =
                  current_tick
                    .saturating_add(1);
              }
            }
          );
        }

        || ()
      }
    );
  }

  {
    let due_notification_config =
      due_notification_config.clone();
    let due_notification_sent =
      due_notification_sent.clone();
    let due_notification_permission =
      due_notification_permission
        .clone();
    let facet_tasks =
      facet_tasks.clone();
    let calendar_config =
      calendar_config.clone();

    use_effect_with(
      (
        (*due_notification_config)
          .clone(),
        (*facet_tasks).clone(),
        (*calendar_config).clone(),
        *due_notification_permission
      ),
      move |(
        config,
        tasks_snapshot,
        calendar_config_snapshot,
        permission
      )| {
        let mut interval =
          None::<Interval>;

        if !config.enabled {
          tracing::debug!(
            "due notification \
             scheduler disabled"
          );
        } else {
          if *permission
            != DueNotificationPermission::Granted
          {
            tracing::warn!(
              permission = ?permission,
              "due notifications \
               enabled without granted \
               browser notification \
               permission"
            );
          }

          let due_notification_sent =
            due_notification_sent
              .clone();
          let due_notification_permission =
            due_notification_permission
              .clone();
          let config =
            config.clone();
          let tasks_snapshot =
            tasks_snapshot.clone();
          let timezone =
            resolve_calendar_timezone(
              calendar_config_snapshot
            );

          let run_check = {
            let due_notification_sent =
              due_notification_sent
                .clone();
            let due_notification_permission =
              due_notification_permission
                .clone();
            let config =
              config.clone();
            let tasks_snapshot =
              tasks_snapshot.clone();
            move || {
              if *due_notification_permission
                != DueNotificationPermission::Granted
              {
                return;
              }

              let now_utc = Utc::now();
              let pending_keys =
                (*due_notification_sent)
                  .clone();
              let events =
                collect_due_notification_events(
                  &tasks_snapshot,
                  timezone,
                  &config,
                  &pending_keys,
                  now_utc
                );

              if events.is_empty() {
                return;
              }

              tracing::info!(
                event_count = events.len(),
                "due notification scan \
                 produced events"
              );

              let mut next_sent =
                pending_keys;
              for event in events {
                if emit_due_notification(
                  &event.title,
                  &event.body
                ) {
                  next_sent
                    .insert(event.key);
                }
              }

              if next_sent
                != *due_notification_sent
              {
                due_notification_sent
                  .set(next_sent);
              }
            }
          };

          run_check();
          interval = Some(Interval::new(
            30_000, run_check
          ));
        }

        move || drop(interval)
      }
    );
  }

  let task_visible_tasks = {
    let query = (*search).clone();
    filter_visible_tasks(
      &tasks,
      &active_view,
      &query,
      active_project.as_deref(),
      active_tag.as_deref(),
      all_filter_completion.as_str(),
      all_filter_project.as_deref(),
      all_filter_tag.as_deref(),
      all_filter_priority.as_str(),
      all_filter_due.as_str()
    )
  };
  let tag_colors =
    build_tag_color_map(&tag_schema);
  let kanban_board_color_map =
    build_kanban_board_color_map(
      &kanban_boards
    );
  let external_calendar_color_map =
    build_external_calendar_color_map(
      &external_calendars
    );
  let kanban_columns =
    kanban_columns_from_schema(
      &tag_schema
    );
  let default_kanban_lane =
    kanban_columns
      .first()
      .cloned()
      .unwrap_or_else(|| {
        "todo".to_string()
      });

  let kanban_visible_tasks = {
    let base = filter_visible_tasks(
      &tasks,
      "kanban",
      "",
      None,
      None,
      all_filter_completion.as_str(),
      all_filter_project.as_deref(),
      all_filter_tag.as_deref(),
      all_filter_priority.as_str(),
      all_filter_due.as_str()
    );

    if let Some(board_id) =
      (*active_kanban_board).clone()
    {
      base
        .into_iter()
        .filter(|task| {
          task_has_tag_value(
            &task.tags,
            BOARD_TAG_KEY,
            &board_id
          )
        })
        .collect()
    } else {
      Vec::new()
    }
  };

  let selected_task = (*selected)
    .and_then(|id| {
      task_visible_tasks
        .iter()
        .find(|task| task.uuid == id)
        .cloned()
    });

  let project_facets =
    build_project_facets(&facet_tasks);
  let tag_facets =
    build_tag_facets(&facet_tasks);
  let calendar_timezone =
    resolve_calendar_timezone(
      &calendar_config
    );
  let calendar_week_start =
    calendar_week_start_day(
      &calendar_config
        .policies
        .week_start
    );
  let calendar_due_tasks =
    collect_calendar_due_tasks(
      &facet_tasks,
      calendar_timezone,
      &calendar_config,
      &kanban_board_color_map,
      &external_calendar_color_map
    );
  let calendar_period_stats =
    summarize_calendar_period(
      &calendar_due_tasks,
      *calendar_view,
      *calendar_focus_date,
      calendar_week_start,
      &calendar_config
    );
  let calendar_period_tasks =
    collect_calendar_period_tasks(
      &calendar_due_tasks,
      *calendar_view,
      *calendar_focus_date,
      calendar_week_start,
      &calendar_config
    );
  let calendar_title =
    calendar_title_for_view(
      *calendar_view,
      *calendar_focus_date,
      calendar_week_start
    );

  let on_nav = {
    let active_view =
      active_view.clone();
    let search = search.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    let active_project =
      active_project.clone();
    let active_tag = active_tag.clone();
    let all_filter_project =
      all_filter_project.clone();
    let all_filter_tag =
      all_filter_tag.clone();
    let all_filter_completion =
      all_filter_completion.clone();
    let all_filter_priority =
      all_filter_priority.clone();
    let all_filter_due =
      all_filter_due.clone();
    let refresh_tick =
      refresh_tick.clone();
    Callback::from(
      move |view: String| {
        if view != "all" {
          search.set(String::new());
        }
        active_view.set(view);
        selected.set(None);
        bulk_selected
          .set(BTreeSet::new());
        active_project.set(None);
        active_tag.set(None);
        all_filter_project.set(None);
        all_filter_tag.set(None);
        all_filter_completion
          .set("all".to_string());
        all_filter_priority
          .set("all".to_string());
        all_filter_due
          .set("all".to_string());
        refresh_tick.set(
          (*refresh_tick)
            .saturating_add(1)
        );
      }
    )
  };

  let on_select_tasks_tab = {
    let active_tab = active_tab.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    let dragging_kanban_task =
      dragging_kanban_task.clone();
    let drag_over_kanban_lane =
      drag_over_kanban_lane.clone();
    let refresh_tick =
      refresh_tick.clone();
    Callback::from(move |_| {
      active_tab
        .set("tasks".to_string());
      selected.set(None);
      bulk_selected
        .set(BTreeSet::new());
      dragging_kanban_task.set(None);
      drag_over_kanban_lane.set(None);
      refresh_tick.set(
        (*refresh_tick)
          .saturating_add(1)
      );
    })
  };

  let on_select_kanban_tab = {
    let active_tab = active_tab.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    let refresh_tick =
      refresh_tick.clone();
    Callback::from(move |_| {
      active_tab
        .set("kanban".to_string());
      selected.set(None);
      bulk_selected
        .set(BTreeSet::new());
      refresh_tick.set(
        (*refresh_tick)
          .saturating_add(1)
      );
    })
  };

  let on_select_calendar_tab = {
    let active_tab = active_tab.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    let dragging_kanban_task =
      dragging_kanban_task.clone();
    let drag_over_kanban_lane =
      drag_over_kanban_lane.clone();
    let refresh_tick =
      refresh_tick.clone();
    Callback::from(move |_| {
      active_tab
        .set("calendar".to_string());
      selected.set(None);
      bulk_selected
        .set(BTreeSet::new());
      dragging_kanban_task.set(None);
      drag_over_kanban_lane.set(None);
      refresh_tick.set(
        (*refresh_tick)
          .saturating_add(1)
      );
    })
  };

  let on_select = {
    let selected = selected.clone();
    Callback::from(move |id: Uuid| {
      selected.set(Some(id))
    })
  };

  let on_toggle_select = {
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(move |id: Uuid| {
      let mut next =
        (*bulk_selected).clone();
      if next.contains(&id) {
        next.remove(&id);
      } else {
        next.insert(id);
      }
      bulk_selected.set(next);
    })
  };

  let on_choose_project = {
    let active_project =
      active_project.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(
      move |project: Option<String>| {
        active_project.set(project);
        selected.set(None);
        bulk_selected
          .set(BTreeSet::new());
      }
    )
  };

  let on_choose_tag = {
    let active_tag = active_tag.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(
      move |tag: Option<String>| {
        active_tag.set(tag);
        selected.set(None);
        bulk_selected
          .set(BTreeSet::new());
      }
    )
  };

  let on_all_completion_change = {
    let all_filter_completion =
      all_filter_completion.clone();
    Callback::from(
      move |e: web_sys::Event| {
        if let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlSelectElement
          >()
        {
          all_filter_completion
            .set(input.value());
        } else {
          tracing::warn!(
            "all completion filter event \
             had non-select target"
          );
        }
      }
    )
  };

  let on_all_project_change = {
    let all_filter_project =
      all_filter_project.clone();
    Callback::from(
      move |e: web_sys::Event| {
        if let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlSelectElement
          >()
        {
          let value = input.value();
          if value.is_empty() {
            all_filter_project
              .set(None);
          } else {
            all_filter_project
              .set(Some(value));
          }
        } else {
          tracing::warn!(
            "all project filter event had \
             non-select target"
          );
        }
      }
    )
  };

  let on_all_tag_change = {
    let all_filter_tag =
      all_filter_tag.clone();
    Callback::from(
      move |e: web_sys::Event| {
        if let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlSelectElement
          >()
        {
          let value = input.value();
          if value.is_empty() {
            all_filter_tag.set(None);
          } else {
            all_filter_tag.set(Some(value));
          }
        } else {
          tracing::warn!(
            "all tag filter event had \
             non-select target"
          );
        }
      }
    )
  };

  let on_all_filters_clear = {
    let all_filter_project =
      all_filter_project.clone();
    let all_filter_tag =
      all_filter_tag.clone();
    let all_filter_completion =
      all_filter_completion.clone();
    let all_filter_priority =
      all_filter_priority.clone();
    let all_filter_due =
      all_filter_due.clone();
    Callback::from(move |_| {
      all_filter_project.set(None);
      all_filter_tag.set(None);
      all_filter_completion
        .set("all".to_string());
      all_filter_priority
        .set("all".to_string());
      all_filter_due
        .set("all".to_string());
    })
  };

  let on_all_priority_change = {
    let all_filter_priority =
      all_filter_priority.clone();
    Callback::from(
      move |e: web_sys::Event| {
        if let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlSelectElement
          >()
        {
          all_filter_priority
            .set(input.value());
        } else {
          tracing::warn!(
            "all priority filter event \
             had non-select target"
          );
        }
      }
    )
  };

  let on_all_due_change = {
    let all_filter_due =
      all_filter_due.clone();
    Callback::from(
      move |e: web_sys::Event| {
        if let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlSelectElement
          >()
        {
          all_filter_due.set(input.value());
        } else {
          tracing::warn!(
            "all due filter event had \
             non-select target"
          );
        }
      }
    )
  };

  let on_calendar_set_view = {
    let calendar_view =
      calendar_view.clone();
    Callback::from(
      move |view: CalendarViewMode| {
        tracing::info!(
          view = %view.as_key(),
          "calendar view changed"
        );
        calendar_view.set(view);
      }
    )
  };

  let on_calendar_prev = {
    let calendar_focus_date =
      calendar_focus_date.clone();
    let calendar_view =
      calendar_view.clone();
    let week_start =
      calendar_week_start;
    Callback::from(move |_| {
      let next = shift_calendar_focus(
        *calendar_focus_date,
        *calendar_view,
        -1,
        week_start
      );
      tracing::debug!(
        from = %calendar_focus_date.format("%Y-%m-%d"),
        to = %next.format("%Y-%m-%d"),
        view = %(*calendar_view).as_key(),
        "calendar moved backward"
      );
      calendar_focus_date.set(next);
    })
  };

  let on_calendar_next = {
    let calendar_focus_date =
      calendar_focus_date.clone();
    let calendar_view =
      calendar_view.clone();
    let week_start =
      calendar_week_start;
    Callback::from(move |_| {
      let next = shift_calendar_focus(
        *calendar_focus_date,
        *calendar_view,
        1,
        week_start
      );
      tracing::debug!(
        from = %calendar_focus_date.format("%Y-%m-%d"),
        to = %next.format("%Y-%m-%d"),
        view = %(*calendar_view).as_key(),
        "calendar moved forward"
      );
      calendar_focus_date.set(next);
    })
  };

  let on_calendar_today = {
    let calendar_focus_date =
      calendar_focus_date.clone();
    let calendar_timezone =
      calendar_timezone;
    Callback::from(move |_| {
      let today = today_in_timezone(
        calendar_timezone
      );
      tracing::info!(
        today = %today.format("%Y-%m-%d"),
        timezone = %calendar_timezone,
        "calendar focus reset to today"
      );
      calendar_focus_date.set(today);
    })
  };

  let on_calendar_navigate = {
    let calendar_focus_date =
      calendar_focus_date.clone();
    let calendar_view =
      calendar_view.clone();
    Callback::from(
      move |(day, view): (
        NaiveDate,
        CalendarViewMode
      )| {
        calendar_focus_date.set(day);
        calendar_view.set(view);
      }
    )
  };

  let on_open_add_external_calendar = {
    let external_calendar_modal =
      external_calendar_modal.clone();
    Callback::from(move |_| {
      external_calendar_modal.set(Some(
          ExternalCalendarModalState {
            mode:
              ExternalCalendarModalMode::Add,
            source:
              new_external_calendar_source(),
            error: None,
          },
        ));
    })
  };

  let on_open_edit_external_calendar = {
    let external_calendar_modal =
      external_calendar_modal.clone();
    Callback::from(
        move |source: ExternalCalendarSource| {
          external_calendar_modal.set(Some(
            ExternalCalendarModalState {
              mode:
                ExternalCalendarModalMode::Edit,
              source,
              error: None,
            },
          ));
        },
      )
  };

  let on_close_external_calendar_modal = {
    let external_calendar_modal =
      external_calendar_modal.clone();
    Callback::from(move |_| {
      external_calendar_modal.set(None);
    })
  };

  let on_submit_external_calendar = {
    let external_calendars =
      external_calendars.clone();
    let external_calendar_modal =
      external_calendar_modal.clone();
    Callback::from(
        move |modal_state: ExternalCalendarModalState| {
          let mut source =
            modal_state.source.clone();

          if source
            .name
            .trim()
            .is_empty()
          {
            let mut next = modal_state;
            next.error = Some(
              "Calendar name is required."
                .to_string(),
            );
            external_calendar_modal
              .set(Some(next));
            return;
          }

          if source
            .location
            .trim()
            .is_empty()
          {
            let mut next = modal_state;
            next.error = Some(
              "Calendar URL is required."
                .to_string(),
            );
            external_calendar_modal
              .set(Some(next));
            return;
          }

          source.name = source
            .name
            .trim()
            .to_string();
          source.location = source
            .location
            .trim()
            .to_string();
          if source
            .color
            .trim()
            .is_empty()
          {
            source.color =
              "#d64545".to_string();
          }

          let mut next_sources =
            (*external_calendars).clone();
          match modal_state.mode {
            | ExternalCalendarModalMode::Add => {
              next_sources.push(source);
            }
            | ExternalCalendarModalMode::Edit => {
              if let Some(existing) =
                next_sources.iter_mut().find(
                  |existing| {
                    existing.id
                      == source.id
                  },
                )
              {
                *existing = source;
              }
            }
          }

          external_calendars
            .set(next_sources);
          external_calendar_modal
            .set(None);
        },
      )
  };

  let on_delete_external_calendar = {
    let external_calendars =
      external_calendars.clone();
    let external_calendar_delete_modal =
      external_calendar_delete_modal
        .clone();
    Callback::from(
      move |calendar_id: String| {
        let Some(source) =
          external_calendars
            .iter()
            .find(|source| {
              source.id == calendar_id
            })
            .cloned()
        else {
          tracing::warn!(
            calendar_id = %calendar_id,
            "delete requested for \
             unknown external \
             calendar"
          );
          return;
        };

        tracing::info!(
          calendar_id = %source.id,
          name = %source.name,
          "opened external calendar \
           delete modal"
        );
        external_calendar_delete_modal
          .set(Some(
            ExternalCalendarDeleteState {
              id:   source.id,
              name: source.name
            },
          ));
      }
    )
  };

  let on_close_external_calendar_delete_modal =
    {
      let external_calendar_delete_modal =
        external_calendar_delete_modal
          .clone();
      Callback::from(move |_| {
        external_calendar_delete_modal
          .set(None);
      })
    };

  let on_confirm_delete_external_calendar =
    {
      let external_calendars =
        external_calendars.clone();
      let external_calendar_delete_modal =
        external_calendar_delete_modal
          .clone();
      Callback::from(
        move |calendar_id: String| {
          let mut next_sources =
            (*external_calendars).clone();
          next_sources.retain(|source| {
            source.id != calendar_id
          });
          tracing::warn!(
            calendar_id = %calendar_id,
            remaining_sources =
              next_sources.len(),
            "deleted external calendar \
             source"
          );
          external_calendars
            .set(next_sources);
          external_calendar_delete_modal
            .set(None);
        }
      )
    };

  let on_sync_external_calendar = {
    let external_calendars =
      external_calendars.clone();
    let external_calendar_busy =
      external_calendar_busy.clone();
    let external_calendar_last_sync =
      external_calendar_last_sync
        .clone();
    let refresh_tick =
      refresh_tick.clone();
    Callback::from(
      move |calendar_id: String| {
        if *external_calendar_busy {
          return;
        }
        let Some(source) =
          external_calendars
            .iter()
            .find(|source| {
              source.id == calendar_id
            })
            .cloned()
        else {
          return;
        };
        if source.imported_ics_file {
          external_calendar_last_sync
            .set(Some(format!(
              "{} was imported from an \
               ICS file. Re-import the \
               file to update it.",
              source.name
            )));
          return;
        }

        external_calendar_busy
          .set(true);
        let external_calendar_busy =
          external_calendar_busy
            .clone();
        let external_calendar_last_sync =
          external_calendar_last_sync
            .clone();
        let refresh_tick =
          refresh_tick.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match invoke_tauri::<
              ExternalCalendarSyncResult,
              _
            >("external_calendar_sync", &source)
            .await
            {
              | Ok(result) => {
                external_calendar_last_sync
                  .set(Some(format!(
                    "Synced {}: +{} / ~{} / -{}",
                    source.name,
                    result.created,
                    result.updated,
                    result.deleted
                  )));
                refresh_tick.set(
                  (*refresh_tick)
                    .saturating_add(1),
                );
              }
              | Err(err) => {
                external_calendar_last_sync
                  .set(Some(format!(
                    "Sync failed for {}: {}",
                    source.name, err
                  )));
              }
            }
            external_calendar_busy
              .set(false);
          });
      }
    )
  };

  let on_sync_all_external_calendars = {
    let external_calendars =
      external_calendars.clone();
    let external_calendar_busy =
      external_calendar_busy.clone();
    let external_calendar_last_sync =
      external_calendar_last_sync
        .clone();
    let refresh_tick =
      refresh_tick.clone();
    Callback::from(move |_| {
      if *external_calendar_busy {
        return;
      }

      let sources: Vec<
        ExternalCalendarSource
      > = external_calendars
        .iter()
        .filter(|source| {
          source.enabled
            && !source
              .imported_ics_file
        })
        .cloned()
        .collect();
      if sources.is_empty() {
        external_calendar_last_sync
          .set(Some(
            "No enabled network \
             calendars to sync."
              .to_string()
          ));
        return;
      }

      external_calendar_busy.set(true);
      let external_calendar_busy =
        external_calendar_busy.clone();
      let external_calendar_last_sync =
        external_calendar_last_sync
          .clone();
      let refresh_tick =
        refresh_tick.clone();
      wasm_bindgen_futures::spawn_local(
        async move {
          let mut lines = Vec::new();
          for source in sources {
            match invoke_tauri::<
              ExternalCalendarSyncResult,
              _
            >("external_calendar_sync", &source)
            .await
            {
              | Ok(result) => {
                lines.push(format!(
                  "{}: +{} / ~{} / -{}",
                  source.name,
                  result.created,
                  result.updated,
                  result.deleted
                ));
              }
              | Err(err) => {
                lines.push(format!(
                  "{}: failed ({})",
                  source.name, err
                ));
              }
            }
          }

          external_calendar_last_sync
            .set(Some(
              lines.join(" | ")
            ));
          refresh_tick.set(
            (*refresh_tick)
              .saturating_add(1)
          );
          external_calendar_busy
            .set(false);
        }
      );
    })
  };

  let on_import_external_calendar_file =
    {
      let external_calendars =
        external_calendars.clone();
      let external_calendar_busy =
        external_calendar_busy.clone();
      let external_calendar_last_sync =
        external_calendar_last_sync
          .clone();
      let refresh_tick =
        refresh_tick.clone();
      Callback::from(
        move |e: web_sys::Event| {
          if *external_calendar_busy {
            tracing::warn!(
              "ignored ICS import while \
               external calendar sync \
               is busy"
            );
            return;
          }

          let Some(input) =
            e.target_dyn_into::<
              web_sys::HtmlInputElement
            >()
          else {
            tracing::warn!(
              "ICS import event had \
               non-input target"
            );
            return;
          };

          let Some(files) = input.files()
          else {
            tracing::warn!(
              "ICS import requested with \
               no file list"
            );
            return;
          };

          let Some(file) = files.get(0)
          else {
            tracing::warn!(
              "ICS import requested with \
               empty file list"
            );
            return;
          };

          let file_name = file.name();
          let stem = file_name
            .strip_suffix(".ics")
            .unwrap_or(file_name.as_str())
            .trim();
          let base_name =
            if stem.is_empty() {
              "Imported Calendar"
            } else {
              stem
            };

          let sources_snapshot =
            (*external_calendars).clone();
          let mut candidate_name =
            base_name.to_string();
          let mut suffix = 2_u32;
          while sources_snapshot
            .iter()
            .any(|source| {
              source.name.eq_ignore_ascii_case(
                &candidate_name,
              )
            })
          {
            candidate_name = format!(
              "{} {}",
              base_name, suffix
            );
            suffix =
              suffix.saturating_add(1);
          }

          input.set_value("");

          let source =
            ExternalCalendarSource {
              id: Uuid::new_v4()
                .to_string(),
              name: candidate_name,
              color: "#d64545"
                .to_string(),
              location: format!(
                "file://{}",
                file_name
              ),
              refresh_minutes: 0,
              enabled: true,
              imported_ics_file:
                true,
              read_only: true,
              show_reminders: true,
              offline_support: true
            };

          external_calendar_busy
            .set(true);
          let external_calendars =
            external_calendars.clone();
          let external_calendar_busy =
            external_calendar_busy
              .clone();
          let external_calendar_last_sync =
            external_calendar_last_sync
              .clone();
          let refresh_tick =
            refresh_tick.clone();

          wasm_bindgen_futures::spawn_local(async move {
              let ics_text = match wasm_bindgen_futures::JsFuture::from(file.text()).await {
                  | Ok(value) => value.as_string().unwrap_or_default(),
                  | Err(error) => {
                      tracing::error!(error = ?error, file_name = %file_name, "failed reading selected ICS file");
                      external_calendar_last_sync.set(Some(format!("Failed to read {}.", file_name)));
                      external_calendar_busy.set(false);
                      return;
                  }
              };

              if ics_text.trim().is_empty() {
                  external_calendar_last_sync.set(Some(format!("{} is empty.", file_name)));
                  external_calendar_busy.set(false);
                  return;
              }

              let import_args = ExternalCalendarImportArgs {
                  source: source.clone(),
                  ics_text,
              };

              match invoke_tauri::<ExternalCalendarSyncResult, _>(
                  "external_calendar_import_ics",
                  &import_args,
              )
              .await
              {
                  | Ok(result) => {
                      let mut next_sources = (*external_calendars).clone();
                      next_sources.push(source.clone());
                      external_calendars.set(next_sources);
                      external_calendar_last_sync.set(Some(format!(
                          "Imported {}: +{} / ~{} / -{}",
                          source.name, result.created, result.updated, result.deleted
                      )));
                      refresh_tick.set((*refresh_tick).saturating_add(1));
                  }
                  | Err(err) => {
                      tracing::error!(error = %err, file_name = %file_name, "ICS import command failed");
                      external_calendar_last_sync.set(Some(format!(
                          "Import failed for {}: {}",
                          file_name, err
                      )));
                  }
              }

              external_calendar_busy.set(false);
          });
        }
      )
    };

  let on_select_kanban_board = {
    let active_kanban_board =
      active_kanban_board.clone();
    let selected = selected.clone();
    Callback::from(
      move |board_id: String| {
        tracing::info!(
          board_id = %board_id,
          "selected kanban board"
        );
        active_kanban_board
          .set(Some(board_id));
        selected.set(None);
      }
    )
  };

  let on_create_kanban_board = {
    let kanban_create_open =
      kanban_create_open.clone();
    let kanban_create_input =
      kanban_create_input.clone();
    Callback::from(move |_| {
      kanban_create_input
        .set(String::new());
      kanban_create_open.set(true);
    })
  };

  let on_close_create_kanban_board = {
    let kanban_create_open =
      kanban_create_open.clone();
    Callback::from(move |_| {
      kanban_create_open.set(false);
    })
  };

  let on_create_kanban_input = {
    let kanban_create_input =
      kanban_create_input.clone();
    Callback::from(
      move |e: web_sys::InputEvent| {
        let input: web_sys::HtmlInputElement =
          e.target_unchecked_into();
        kanban_create_input
          .set(input.value());
      }
    )
  };

  let on_submit_create_kanban_board = {
    let kanban_boards =
      kanban_boards.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    let kanban_create_open =
      kanban_create_open.clone();
    let kanban_create_input =
      kanban_create_input.clone();
    Callback::from(move |_| {
      let name = (*kanban_create_input)
        .trim()
        .to_string();

      if name.is_empty() {
        tracing::warn!(
          "ignored empty kanban board \
           name"
        );
        return;
      }

      let mut next =
        (*kanban_boards).clone();
      let unique_name =
        make_unique_board_name(
          &next, &name
        );
      let board_id =
        Uuid::new_v4().to_string();
      tracing::info!(
        board_id = %board_id,
        name = %unique_name,
        "creating kanban board"
      );
      let color =
        next_board_color(&next);
      next.push(KanbanBoardDef {
        id: board_id.clone(),
        name: unique_name,
        color
      });

      kanban_boards.set(next);
      active_kanban_board
        .set(Some(board_id));
      kanban_create_open.set(false);
    })
  };

  let on_toggle_kanban_card_density = {
    let kanban_compact_cards =
      kanban_compact_cards.clone();
    Callback::from(move |_| {
      kanban_compact_cards
        .set(!*kanban_compact_cards);
    })
  };

  let on_open_rename_kanban_board = {
    let kanban_boards =
      kanban_boards.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    let kanban_rename_input =
      kanban_rename_input.clone();
    let kanban_rename_open =
      kanban_rename_open.clone();
    Callback::from(move |_| {
      let Some(board_id) =
        (*active_kanban_board).clone()
      else {
        tracing::warn!(
          "rename board clicked with \
           no active board"
        );
        return;
      };

      let Some(current) =
        (*kanban_boards)
          .iter()
          .find(|board| {
            board.id == board_id
          })
          .cloned()
      else {
        tracing::warn!(
          %board_id,
          "active board not found \
           for rename modal"
        );
        return;
      };

      kanban_rename_input
        .set(current.name);
      kanban_rename_open.set(true);
    })
  };

  let on_close_rename_kanban_board = {
    let kanban_rename_open =
      kanban_rename_open.clone();
    Callback::from(move |_| {
      kanban_rename_open.set(false);
    })
  };

  let on_rename_kanban_input = {
    let kanban_rename_input =
      kanban_rename_input.clone();
    Callback::from(
      move |e: web_sys::InputEvent| {
        let input: web_sys::HtmlInputElement =
          e.target_unchecked_into();
        kanban_rename_input
          .set(input.value());
      }
    )
  };

  let on_submit_rename_kanban_board = {
    let kanban_boards =
      kanban_boards.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    let kanban_rename_open =
      kanban_rename_open.clone();
    let kanban_rename_input =
      kanban_rename_input.clone();
    Callback::from(move |_| {
      let Some(board_id) =
        (*active_kanban_board).clone()
      else {
        tracing::warn!(
          "rename board clicked with \
           no active board"
        );
        return;
      };

      let name = (*kanban_rename_input)
        .trim()
        .to_string();

      if name.is_empty() {
        tracing::warn!(
          %board_id,
          "ignored empty rename \
           request"
        );
        return;
      }

      let mut next =
        (*kanban_boards).clone();
      let unique_name =
        make_unique_board_name_except(
          &next, &name, &board_id
        );
      for board in &mut next {
        if board.id == board_id {
          board.name =
            unique_name.clone();
        }
      }

      tracing::info!(
        %board_id,
        name = %unique_name,
        "renamed kanban board"
      );
      kanban_boards.set(next);
      kanban_rename_open.set(false);
    })
  };

  let on_delete_kanban_board = {
    let kanban_boards =
      kanban_boards.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    let facet_tasks =
      facet_tasks.clone();
    let refresh_tick =
      refresh_tick.clone();
    Callback::from(move |_| {
      let Some(board_id) =
        (*active_kanban_board).clone()
      else {
        tracing::warn!(
          "delete board clicked with \
           no active board"
        );
        return;
      };

      let Some(board) =
        (*kanban_boards)
          .iter()
          .find(|entry| {
            entry.id == board_id
          })
          .cloned()
      else {
        tracing::warn!(
          %board_id,
          "active board not found \
           for deletion"
        );
        return;
      };

      let confirmed = web_sys::window()
        .and_then(|window| {
          window
            .confirm_with_message(
              &format!(
                "Delete board \
                 '{}'?\nThis removes \
                 board assignment \
                 from pending tasks \
                 using this board.",
                board.name
              )
            )
            .ok()
        })
        .unwrap_or(false);

      if !confirmed {
        tracing::info!(
          %board_id,
          "board deletion canceled"
        );
        return;
      }

      let mut next_boards =
        (*kanban_boards).clone();
      next_boards.retain(|entry| {
        entry.id != board_id
      });

      let next_active = next_boards
        .first()
        .map(|entry| entry.id.clone());
      tracing::warn!(
        %board_id,
        next_active = ?next_active,
        "deleted kanban board"
      );
      kanban_boards.set(next_boards);
      active_kanban_board
        .set(next_active);

      let tasks_to_clean: Vec<TaskDto> =
        (*facet_tasks)
          .iter()
          .filter(|task| {
            matches!(
              task.status,
              TaskStatus::Pending
                | TaskStatus::Waiting
            ) && task_has_tag_value(
              &task.tags,
              BOARD_TAG_KEY,
              &board_id
            )
          })
          .cloned()
          .collect();

      let refresh_tick =
        refresh_tick.clone();
      wasm_bindgen_futures::spawn_local(
        async move {
          for task in tasks_to_clean {
            let mut next_tags =
              task.tags.clone();
            remove_board_tag_for_id(
              &mut next_tags,
              &board_id
            );

            let update =
              TaskUpdateArgs {
                uuid:  task.uuid,
                patch: TaskPatch {
                  tags: Some(next_tags),
                  ..TaskPatch::default()
                }
              };

            if let Err(err) = invoke_tauri::<TaskDto, _>("task_update", &update).await {
                        tracing::error!(error = %err, task = %task.uuid, board_id = %board_id, "failed clearing deleted board tag");
                    }
          }

          refresh_tick.set(
            (*refresh_tick)
              .saturating_add(1)
          );
        }
      );
    })
  };

  let on_add_click = {
    let modal_state =
      modal_state.clone();
    let modal_busy = modal_busy.clone();
    let modal_submit_seq =
      modal_submit_seq.clone();
    let tag_schema = tag_schema.clone();
    let active_tab = active_tab.clone();
    let active_kanban_board =
      active_kanban_board.clone();
    Callback::from(move |_| {
      let (picker_key, picker_value) =
        tag_schema.default_picker();
      let draft_board_id =
        if *active_tab == "kanban" {
          (*active_kanban_board).clone()
        } else {
          None
        };
      let lock_board_selection =
        *active_tab == "kanban"
          && draft_board_id.is_some();
      modal_busy.set(false);
      modal_submit_seq.set(
        (*modal_submit_seq)
          .wrapping_add(1)
      );
      modal_state.set(Some(
        ModalState {
          mode: ModalMode::Add,
          draft_title: String::new(),
          draft_desc: String::new(),
          draft_project: String::new(),
          draft_board_id,
          lock_board_selection,
          draft_custom_tag: String::new(
          ),
          draft_tags: vec![],
          picker_key,
          picker_value,
          draft_due: String::new(),
          recurrence_pattern: "none"
            .to_string(),
          recurrence_time: String::new(
          ),
          recurrence_days: vec![],
          recurrence_months: vec![],
          recurrence_month_day:
            String::new(),
          allow_recurrence: true,
          error: None
        }
      ));
      ui_debug(
        "action.add_modal.open",
        "clicked Add Task"
      );
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

  let on_due_enabled = {
    let due_notification_config =
      due_notification_config.clone();
    let due_notification_permission =
      due_notification_permission
        .clone();
    Callback::from(
      move |e: web_sys::Event| {
        let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlInputElement
          >()
        else {
          tracing::warn!(
            "due notification enable \
             event had non-input \
             target"
          );
          return;
        };

        let enabled = input.checked();
        let mut next =
          (*due_notification_config)
            .clone();
        next.enabled = enabled;
        if !enabled {
          next.pre_notify_enabled = false;
        }
        tracing::info!(
          enabled = next.enabled,
          pre_notify_enabled =
            next.pre_notify_enabled,
          pre_notify_minutes =
            next.pre_notify_minutes,
          "updated due notification \
           config from enable toggle"
        );
        due_notification_config
          .set(next);

        if enabled
          && *due_notification_permission
            == DueNotificationPermission::Default
        {
          request_due_notification_permission(
            due_notification_permission
              .clone(),
          );
        }
      }
    )
  };

  let on_due_pre_enabled = {
    let due_notification_config =
      due_notification_config.clone();
    Callback::from(
      move |e: web_sys::Event| {
        let Some(input) =
          e.target_dyn_into::<
            web_sys::HtmlInputElement
          >()
        else {
          tracing::warn!(
            "pre-notify enable event \
             had non-input target"
          );
          return;
        };

        let enabled = input.checked();
        let mut next =
          (*due_notification_config)
            .clone();
        next.pre_notify_enabled =
          enabled && next.enabled;
        tracing::info!(
          pre_notify_enabled =
            next.pre_notify_enabled,
          "updated due notification \
           pre-notify toggle"
        );
        due_notification_config
          .set(next);
      }
    )
  };

  let on_due_pre_minutes_input = {
    let due_notification_config =
      due_notification_config.clone();
    Callback::from(
      move |e: web_sys::InputEvent| {
        let input: web_sys::HtmlInputElement =
          e.target_unchecked_into();
        let parsed = input
          .value()
          .trim()
          .parse::<u32>()
          .ok()
          .unwrap_or(15)
          .max(1)
          .min(43_200);
        let mut next =
          (*due_notification_config)
            .clone();
        next.pre_notify_minutes = parsed;
        tracing::debug!(
          pre_notify_minutes =
            parsed,
          "updated due notification \
           pre-notify minutes"
        );
        due_notification_config
          .set(next);
      }
    )
  };

  let on_request_due_permission = {
    let due_notification_permission =
      due_notification_permission
        .clone();
    Callback::from(move |_| {
      request_due_notification_permission(
        due_notification_permission
          .clone(),
      );
    })
  };

  let on_window_minimize =
    Callback::from(move |_| {
      wasm_bindgen_futures::spawn_local(
        async move {
          if let Err(err) =
            invoke_tauri::<(), _>(
              "window_minimize",
              &()
            )
            .await
          {
            tracing::error!(
              error = %err,
              "window minimize failed"
            );
          }
        }
      );
    });

  let on_window_toggle_maximize =
    Callback::from(move |_| {
      wasm_bindgen_futures::spawn_local(
        async move {
          if let Err(err) =
            invoke_tauri::<(), _>(
              "window_toggle_maximize",
              &()
            )
            .await
          {
            tracing::error!(
              error = %err,
              "window toggle maximize failed"
            );
          }
        }
      );
    });

  let on_window_close =
    Callback::from(move |_| {
      wasm_bindgen_futures::spawn_local(
        async move {
          if let Err(err) =
            invoke_tauri::<(), _>(
              "window_close",
              &()
            )
            .await
          {
            tracing::error!(
              error = %err,
              "window close failed"
            );
          }
        }
      );
    });

  let on_done = {
    let refresh_tick =
      refresh_tick.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(move |uuid: Uuid| {
      let refresh_tick =
        refresh_tick.clone();
      let selected = selected.clone();
      let bulk_selected =
        bulk_selected.clone();

      wasm_bindgen_futures::spawn_local(
        async move {
          let arg = TaskIdArg {
            uuid
          };
          match invoke_tauri::<TaskDto, _>("task_done", &arg).await {
                    Ok(_) => {
                        selected.set(None);
                        bulk_selected.set(BTreeSet::new());
                        refresh_tick.set((*refresh_tick).saturating_add(1));
                    }
                    Err(err) => tracing::error!(error = %err, "task_done failed"),
                }
        }
      );
    })
  };

  let on_delete = {
    let refresh_tick =
      refresh_tick.clone();
    let selected = selected.clone();
    let bulk_selected =
      bulk_selected.clone();
    Callback::from(move |uuid: Uuid| {
      let refresh_tick =
        refresh_tick.clone();
      let selected = selected.clone();
      let bulk_selected =
        bulk_selected.clone();

      wasm_bindgen_futures::spawn_local(
        async move {
          let arg = TaskIdArg {
            uuid
          };
          match invoke_tauri::<(), _>(
            "task_delete",
            &arg
          )
          .await
          {
            | Ok(()) => {
              selected.set(None);
              bulk_selected
                .set(BTreeSet::new());
              refresh_tick.set(
                (*refresh_tick)
                  .saturating_add(1)
              );
            }
            | Err(err) => {
              tracing::error!(error = %err, "task_delete failed")
            }
          }
        }
      );
    })
  };

  let on_kanban_move = {
    let tasks = tasks.clone();
    let kanban_columns =
      kanban_columns.clone();
    let default_kanban_lane =
      default_kanban_lane.clone();
    let refresh_tick =
      refresh_tick.clone();
    let dragging_kanban_task =
      dragging_kanban_task.clone();
    let drag_over_kanban_lane =
      drag_over_kanban_lane.clone();
    Callback::from(
      move |(uuid, lane): (
        Uuid,
        String
      )| {
        dragging_kanban_task.set(None);
        drag_over_kanban_lane.set(None);

        let target_lane =
          if kanban_columns.iter().any(
            |column| column == &lane
          ) {
            lane
          } else {
            tracing::warn!(
              lane = %lane,
              fallback = %default_kanban_lane,
              "unknown kanban lane during \
               move; falling back to \
               default lane"
            );
            default_kanban_lane.clone()
          };

        let Some(task) = (*tasks)
          .iter()
          .find(|task| {
            task.uuid == uuid
          })
          .cloned()
        else {
          tracing::warn!(
            %uuid,
            "kanban move ignored because \
             task is not in current \
             snapshot"
          );
          return;
        };

        if !matches!(
          task.status,
          TaskStatus::Pending
            | TaskStatus::Waiting
        ) {
          tracing::warn!(
            %uuid,
            status = ?task.status,
            "kanban move ignored for \
             non-pending task"
          );
          return;
        }

        let mut next_tags =
          task.tags.clone();
        remove_tags_for_key(
          &mut next_tags,
          KANBAN_TAG_KEY
        );
        push_tag_unique(
          &mut next_tags,
          format!(
            "{KANBAN_TAG_KEY}:\
             {target_lane}"
          )
        );

        tracing::info!(
          %uuid,
          lane = %target_lane,
          tag_count = next_tags.len(),
          "moving task in kanban by \
           rewriting lane tag"
        );

        let update = TaskUpdateArgs {
          uuid,
          patch: TaskPatch {
            tags: Some(next_tags),
            ..TaskPatch::default()
          }
        };

        let refresh_tick =
          refresh_tick.clone();
        wasm_bindgen_futures::spawn_local(
          async move {
            match invoke_tauri::<TaskDto, _>(
              "task_update",
              &update
            )
            .await
            {
              | Ok(_) => {
                refresh_tick.set(
                  (*refresh_tick)
                    .saturating_add(1)
                );
              }
              | Err(err) => tracing::error!(error = %err, %uuid, "kanban move task_update failed")
            }
          }
        );
      }
    )
  };

  let on_kanban_drag_start = {
    let dragging_kanban_task =
      dragging_kanban_task.clone();
    Callback::from(move |uuid: Uuid| {
      tracing::debug!(
        %uuid,
        "kanban drag start"
      );
      dragging_kanban_task
        .set(Some(uuid));
    })
  };

  let on_kanban_drag_end = {
    let dragging_kanban_task =
      dragging_kanban_task.clone();
    let drag_over_kanban_lane =
      drag_over_kanban_lane.clone();
    Callback::from(move |_| {
      tracing::debug!(
        "kanban drag end"
      );
      dragging_kanban_task.set(None);
      drag_over_kanban_lane.set(None);
    })
  };

  let on_kanban_drag_over_lane = {
    let drag_over_kanban_lane =
      drag_over_kanban_lane.clone();
    Callback::from(
      move |lane: String| {
        if (*drag_over_kanban_lane)
          .as_deref()
          != Some(lane.as_str())
        {
          tracing::debug!(
            lane = %lane,
            "kanban drag over lane"
          );
          drag_over_kanban_lane
            .set(Some(lane));
        }
      }
    )
  };

  let on_bulk_done = {
    let bulk_selected =
      bulk_selected.clone();
    let refresh_tick =
      refresh_tick.clone();
    let selected = selected.clone();
    Callback::from(move |_| {
      let ids: Vec<Uuid> =
        (*bulk_selected)
          .iter()
          .copied()
          .collect();
      if ids.is_empty() {
        return;
      }

      let bulk_selected =
        bulk_selected.clone();
      let refresh_tick =
        refresh_tick.clone();
      let selected = selected.clone();

      wasm_bindgen_futures::spawn_local(
        async move {
          for uuid in ids {
            let arg = TaskIdArg {
              uuid
            };
            if let Err(err) = invoke_tauri::<TaskDto, _>("task_done", &arg).await {
                        tracing::error!(error = %err, %uuid, "bulk task_done failed");
                    }
          }

          selected.set(None);
          bulk_selected
            .set(BTreeSet::new());
          refresh_tick.set(
            (*refresh_tick)
              .saturating_add(1)
          );
        }
      );
    })
  };

  let on_bulk_delete = {
    let bulk_selected =
      bulk_selected.clone();
    let refresh_tick =
      refresh_tick.clone();
    let selected = selected.clone();
    Callback::from(move |_| {
      let ids: Vec<Uuid> =
        (*bulk_selected)
          .iter()
          .copied()
          .collect();
      if ids.is_empty() {
        return;
      }

      let bulk_selected =
        bulk_selected.clone();
      let refresh_tick =
        refresh_tick.clone();
      let selected = selected.clone();

      wasm_bindgen_futures::spawn_local(
        async move {
          for uuid in ids {
            let arg = TaskIdArg {
              uuid
            };
            if let Err(err) =
              invoke_tauri::<(), _>(
                "task_delete",
                &arg
              )
              .await
            {
              tracing::error!(error = %err, %uuid, "bulk task_delete failed");
            }
          }

          selected.set(None);
          bulk_selected
            .set(BTreeSet::new());
          refresh_tick.set(
            (*refresh_tick)
              .saturating_add(1)
          );
        }
      );
    })
  };

  let on_edit = {
    let modal_state =
      modal_state.clone();
    let modal_busy = modal_busy.clone();
    let modal_submit_seq =
      modal_submit_seq.clone();
    let tag_schema = tag_schema.clone();
    let kanban_boards =
      kanban_boards.clone();
    let external_calendars =
      external_calendars.clone();
    Callback::from(
      move |task: TaskDto| {
        let (picker_key, picker_value) =
          tag_schema.default_picker();
        let draft_board_id =
          board_id_from_task_tags(
            &kanban_boards,
            &task.tags
          );
        let (
          recurrence_pattern,
          recurrence_time,
          recurrence_days,
          recurrence_months,
          recurrence_month_day
        ) = recurrence_from_tags(
          &task.tags
        );
        let filtered_tags = task
          .tags
          .iter()
          .cloned()
          .filter(|tag| {
            !tag.starts_with(&format!(
              "{BOARD_TAG_KEY}:"
            )) && !is_recurrence_tag(
              tag
            )
          })
          .collect();
        let allow_recurrence =
          first_tag_value(
            &task.tags,
            CAL_SOURCE_TAG_KEY,
          )
          .and_then(|calendar_id| {
            external_calendars
              .iter()
              .find(|source| {
                source.id
                  == calendar_id
              })
          })
          .is_none_or(|source| {
            !source
              .imported_ics_file
          });
        modal_busy.set(false);
        modal_submit_seq.set(
          (*modal_submit_seq)
            .wrapping_add(1)
        );
        modal_state.set(Some(
          ModalState {
            mode: ModalMode::Edit(
              task.uuid
            ),
            draft_title: task.title,
            draft_desc: task
              .description,
            draft_project: task
              .project
              .unwrap_or_default(),
            draft_board_id,
            lock_board_selection: false,
            draft_custom_tag:
              String::new(),
            draft_tags: filtered_tags,
            picker_key,
            picker_value,
            draft_due: task
              .due
              .unwrap_or_default(),
            recurrence_pattern,
            recurrence_time,
            recurrence_days,
            recurrence_months,
            recurrence_month_day,
            allow_recurrence,
            error: None
          }
        ));
      }
    )
  };

  let close_modal = {
    let modal_state =
      modal_state.clone();
    let modal_busy = modal_busy.clone();
    let modal_submit_seq =
      modal_submit_seq.clone();
    Callback::from(move |_| {
      modal_busy.set(false);
      modal_submit_seq.set(
        (*modal_submit_seq)
          .wrapping_add(1)
      );
      modal_state.set(None);
      ui_debug(
        "action.modal.cancel",
        "Cancel clicked, closing modal"
      );
    })
  };

  let on_modal_close_click = {
    let close_modal =
      close_modal.clone();
    Callback::from(move |_| {
      close_modal.emit(())
    })
  };

  let on_modal_submit = {
    let modal_state =
      modal_state.clone();
    let refresh_tick =
      refresh_tick.clone();
    let modal_busy = modal_busy.clone();
    let modal_submit_seq =
      modal_submit_seq.clone();
    let kanban_boards =
      kanban_boards.clone();
    let default_kanban_lane =
      default_kanban_lane.clone();
    Callback::from(
      move |state: ModalState| {
        if *modal_busy {
          ui_debug(
            "action.modal.submit.skip",
            "ignored duplicate while \
             busy"
          );
          return;
        }
        modal_busy.set(true);
        let submit_seq =
          (*modal_submit_seq)
            .wrapping_add(1);
        modal_submit_seq
          .set(submit_seq);
        ui_debug(
          "action.modal.submit",
          &format!(
            "mode={}, title_len={}, \
             desc_len={}",
            match state.mode {
              | ModalMode::Add => "add",
              | ModalMode::Edit(_) =>
                "edit",
            },
            state.draft_title.len(),
            state.draft_desc.len()
          )
        );
        let modal_state =
          modal_state.clone();
        let refresh_tick =
          refresh_tick.clone();
        let modal_busy =
          modal_busy.clone();
        let modal_submit_seq =
          modal_submit_seq.clone();
        let kanban_boards =
          kanban_boards.clone();
        let default_kanban_lane =
          default_kanban_lane.clone();

        {
          let modal_state =
            modal_state.clone();
          let modal_busy =
            modal_busy.clone();
          let modal_submit_seq =
            modal_submit_seq.clone();
          let timeout_state =
            state.clone();
          wasm_bindgen_futures::spawn_local(async move {
                    TimeoutFuture::new(8_000).await;
                    if *modal_busy && *modal_submit_seq == submit_seq {
                        let mut next = timeout_state;
                        next.error = Some(
                            "Save timed out waiting for backend response. Check Tauri IPC/capability configuration."
                                .to_string(),
                        );
                        modal_state.set(Some(next));
                        modal_busy.set(false);
                        ui_debug("action.modal.submit.timeout", "save invoke timed out");
                    }
                });
        }

        wasm_bindgen_futures::spawn_local(async move {
                if state.draft_title.trim().is_empty() {
                    let mut next = state.clone();
                    next.error = Some("Title is required.".to_string());
                    modal_state.set(Some(next));
                    modal_busy.set(false);
                    return;
                }

                let board_tag = state
                    .draft_board_id
                    .clone()
                    .and_then(|board_id| {
                        kanban_boards
                            .iter()
                            .find(|board| board.id == board_id)
                            .map(|board| format!("{BOARD_TAG_KEY}:{}", board.id))
                    });

                match state.mode {
                    ModalMode::Add => {
                        let create = TaskCreate {
                            title: state.draft_title.trim().to_string(),
                            description: state.draft_desc.trim().to_string(),
                            project: optional_text(&state.draft_project),
                            tags: collect_tags_for_submit(
                                &state,
                                board_tag.clone(),
                                state
                                  .allow_recurrence,
                                true,
                                &default_kanban_lane,
                            ),
                            priority: None,
                            due: optional_text(&state.draft_due),
                            wait: None,
                            scheduled: None,
                        };

                        ui_debug("invoke.task_add.begin", "calling tauri command task_add");
                        if let Err(err) = invoke_tauri::<TaskDto, _>("task_add", &create).await {
                            tracing::error!(error = %err, "task_add failed");
                            ui_debug("invoke.task_add.error", &err);
                            let mut next = state.clone();
                            next.error = Some(format!("Save failed: {err}"));
                            modal_state.set(Some(next));
                            modal_busy.set(false);
                            modal_submit_seq.set(submit_seq.wrapping_add(1));
                            return;
                        }
                        ui_debug("invoke.task_add.ok", "task_add succeeded");
                    }
                    ModalMode::Edit(uuid) => {
                        let update = TaskUpdateArgs {
                            uuid,
                            patch: TaskPatch {
                                title: Some(state.draft_title.trim().to_string()),
                                description: Some(state.draft_desc.trim().to_string()),
                                project: Some(optional_text(&state.draft_project)),
                                tags: Some(collect_tags_for_submit(
                                    &state,
                                    board_tag,
                                    state
                                      .allow_recurrence,
                                    false,
                                    &default_kanban_lane,
                                )),
                                due: Some(optional_text(&state.draft_due)),
                                ..TaskPatch::default()
                            },
                        };

                        ui_debug(
                            "invoke.task_update.begin",
                            &format!("calling tauri command task_update uuid={uuid}"),
                        );
                        if let Err(err) = invoke_tauri::<TaskDto, _>("task_update", &update).await {
                            tracing::error!(error = %err, "task_update failed");
                            ui_debug("invoke.task_update.error", &err);
                            let mut next = state.clone();
                            next.error = Some(format!("Save failed: {err}"));
                            modal_state.set(Some(next));
                            modal_busy.set(false);
                            modal_submit_seq.set(submit_seq.wrapping_add(1));
                            return;
                        }
                        ui_debug("invoke.task_update.ok", "task_update succeeded");
                    }
                }

                ui_debug("action.modal.close", "save complete, closing modal");
                modal_state.set(None);
                refresh_tick.set((*refresh_tick).saturating_add(1));
                modal_busy.set(false);
                modal_submit_seq.set(submit_seq.wrapping_add(1));
            });
      }
    )
  };

  let bulk_count =
    (*bulk_selected).len();
  let is_any_modal_open = (*modal_state)
    .is_some()
    || *kanban_create_open
    || *kanban_rename_open
    || (*external_calendar_delete_modal)
      .is_some()
    || (*external_calendar_modal)
      .is_some();
  let active_kanban_board_name =
    (*active_kanban_board)
      .as_ref()
      .and_then(|board_id| {
        kanban_boards
          .iter()
          .find(|board| {
            &board.id == board_id
          })
          .map(|board| {
            board.name.clone()
          })
      });

  html! {
      <div class={classes!("app", (*theme).as_class(), is_any_modal_open.then_some("modal-open"))}>
          <WindowChrome
              on_window_minimize={on_window_minimize}
              on_window_toggle_maximize={on_window_toggle_maximize}
              on_window_close={on_window_close}
              title={"Rivet".to_string()}
              icon_src={"/mascot-square.png".to_string()}
              icon_alt={"Rivet mascot".to_string()}
          />
          <WorkspaceTabs
              active_tab={(*active_tab).clone()}
              on_select_tasks_tab={on_select_tasks_tab}
              on_select_kanban_tab={on_select_kanban_tab}
              on_select_calendar_tab={on_select_calendar_tab}
              bulk_count={bulk_count}
              on_bulk_done={on_bulk_done.clone()}
              on_bulk_delete={on_bulk_delete.clone()}
              on_add_click={on_add_click}
              on_toggle_theme={on_toggle_theme}
              theme_toggle_label={(*theme).toggle_label()}
          />

          <div class="main">
              {
                  if *active_tab == "calendar" {
                      html! {
                          <CalendarWorkspace
                              calendar_view={*calendar_view}
                              on_calendar_set_view={on_calendar_set_view.clone()}
                              on_calendar_prev={on_calendar_prev.clone()}
                              on_calendar_today={on_calendar_today.clone()}
                              on_calendar_next={on_calendar_next.clone()}
                              calendar_timezone={calendar_timezone}
                              calendar_focus_date={*calendar_focus_date}
                              calendar_due_tasks={calendar_due_tasks.clone()}
                              external_calendars={(*external_calendars).clone()}
                              external_busy={*external_calendar_busy}
                              external_last_sync={(*external_calendar_last_sync).clone()}
                              on_open_add_external_calendar={on_open_add_external_calendar.clone()}
                              on_sync_all_external_calendars={on_sync_all_external_calendars.clone()}
                              on_import_external_calendar_file={on_import_external_calendar_file.clone()}
                              on_sync_external_calendar={on_sync_external_calendar.clone()}
                              on_open_edit_external_calendar={on_open_edit_external_calendar.clone()}
                              on_delete_external_calendar={on_delete_external_calendar.clone()}
                              calendar_title={calendar_title.clone()}
                              calendar_week_start={calendar_week_start}
                              calendar_config={(*calendar_config).clone()}
                              tag_colors={tag_colors.clone()}
                              on_calendar_navigate={on_calendar_navigate.clone()}
                              calendar_period_stats={calendar_period_stats}
                              calendar_period_tasks={calendar_period_tasks.clone()}
                          />
                      }
                  } else if *active_tab == "kanban" {
                      html! {
                          <KanbanWorkspace
                              kanban_boards={(*kanban_boards).clone()}
                              active_kanban_board_id={(*active_kanban_board).clone()}
                              active_kanban_board_name={active_kanban_board_name.clone()}
                              kanban_compact_cards={*kanban_compact_cards}
                              on_create_kanban_board={on_create_kanban_board}
                              on_open_rename_kanban_board={on_open_rename_kanban_board.clone()}
                              on_delete_kanban_board={on_delete_kanban_board.clone()}
                              on_toggle_card_density={on_toggle_kanban_card_density.clone()}
                              on_select_kanban_board={on_select_kanban_board}
                              kanban_visible_tasks={kanban_visible_tasks.clone()}
                              kanban_columns={kanban_columns.clone()}
                              tag_colors={tag_colors.clone()}
                              dragging_task={*dragging_kanban_task}
                              drag_over_lane={(*drag_over_kanban_lane).clone()}
                              on_kanban_move={on_kanban_move}
                              on_kanban_drag_start={on_kanban_drag_start}
                              on_kanban_drag_end={on_kanban_drag_end}
                              on_kanban_drag_over_lane={on_kanban_drag_over_lane}
                              on_edit={on_edit.clone()}
                              on_done={on_done.clone()}
                              on_delete={on_delete.clone()}
                              completion_value={(*all_filter_completion).clone()}
                              on_completion_change={on_all_completion_change}
                              project_value={(*all_filter_project).clone().unwrap_or_default()}
                              project_items={project_facets.clone()}
                              on_project_change={on_all_project_change}
                              tag_value={(*all_filter_tag).clone().unwrap_or_default()}
                              tag_items={tag_facets.clone()}
                              on_tag_change={on_all_tag_change}
                              priority_value={(*all_filter_priority).clone()}
                              on_priority_change={on_all_priority_change}
                              due_value={(*all_filter_due).clone()}
                              on_due_change={on_all_due_change}
                              on_clear_filters={on_all_filters_clear.clone()}
                          />
                      }
                  } else if *active_view == "settings" {
                      html! {
                          <SettingsWorkspace
                              active_view={(*active_view).clone()}
                              on_nav={on_nav.clone()}
                              tasks_loaded={tasks.len()}
                              bulk_count={bulk_count}
                              due_notifications={(*due_notification_config).clone()}
                              due_permission={*due_notification_permission}
                              on_due_enabled={on_due_enabled.clone()}
                              on_due_pre_enabled={on_due_pre_enabled.clone()}
                              on_due_pre_minutes_input={on_due_pre_minutes_input.clone()}
                              on_request_due_permission={on_request_due_permission.clone()}
                          />
                      }
                  } else {
                      html! {
                          <TasksWorkspace
                              active_view={(*active_view).clone()}
                              on_nav={on_nav.clone()}
                              task_visible_tasks={task_visible_tasks.clone()}
                              tag_colors={tag_colors.clone()}
                              selected={*selected}
                              bulk_selected={(*bulk_selected).clone()}
                              on_select={on_select}
                              on_toggle_select={on_toggle_select}
                              selected_task={selected_task.clone()}
                              active_project={(*active_project).clone()}
                              active_tag={(*active_tag).clone()}
                              project_facets={project_facets.clone()}
                              tag_facets={tag_facets.clone()}
                              on_choose_project={on_choose_project}
                              on_choose_tag={on_choose_tag}
                              search_value={(*search).clone()}
                              on_search_input={{
                                  let search = search.clone();
                                  Callback::from(move |e: web_sys::InputEvent| {
                                      let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                      search.set(input.value());
                                  })
                              }}
                              completion_value={(*all_filter_completion).clone()}
                              on_completion_change={on_all_completion_change}
                              project_value={(*all_filter_project).clone().unwrap_or_default()}
                              on_project_change={on_all_project_change}
                              tag_value={(*all_filter_tag).clone().unwrap_or_default()}
                              on_tag_change={on_all_tag_change}
                              priority_value={(*all_filter_priority).clone()}
                              on_priority_change={on_all_priority_change}
                              due_value={(*all_filter_due).clone()}
                              on_due_change={on_all_due_change}
                              on_clear_filters={on_all_filters_clear.clone()}
                              on_done={on_done.clone()}
                              on_delete={on_delete.clone()}
                              on_edit={on_edit.clone()}
                          />
                      }
                  }
              }
          </div>

          {
              html! {
                  <TaskModal
                      modal_state={modal_state.clone()}
                      modal_busy={*modal_busy}
                      kanban_boards={(*kanban_boards).clone()}
                      tag_schema={(*tag_schema).clone()}
                      tag_colors={tag_colors.clone()}
                      on_modal_submit={on_modal_submit.clone()}
                      on_modal_close_click={on_modal_close_click.clone()}
                  />
              }
          }

          {
              html! {
                  <NewKanbanBoardModal
                      open={*kanban_create_open}
                      input_value={(*kanban_create_input).clone()}
                      on_close={on_close_create_kanban_board.clone()}
                      on_input={on_create_kanban_input}
                      on_submit={on_submit_create_kanban_board}
                  />
              }
          }

          {
              html! {
                  <RenameKanbanBoardModal
                      open={*kanban_rename_open}
                      input_value={(*kanban_rename_input).clone()}
                      on_close={on_close_rename_kanban_board.clone()}
                      on_input={on_rename_kanban_input}
                      on_submit={on_submit_rename_kanban_board}
                  />
              }
          }

          {
              html! {
                  <ExternalCalendarModal
                      modal_state={external_calendar_modal.clone()}
                      busy={*external_calendar_busy}
                      on_close={on_close_external_calendar_modal.clone()}
                      on_submit={on_submit_external_calendar.clone()}
                  />
              }
          }

          {
              html! {
                  <ExternalCalendarDeleteModal
                      modal_state={external_calendar_delete_modal.clone()}
                      on_close={on_close_external_calendar_delete_modal.clone()}
                      on_confirm={on_confirm_delete_external_calendar.clone()}
                  />
              }
          }
      </div>
  }
}
