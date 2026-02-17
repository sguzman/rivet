mod commands;
mod state;

use std::env;

use anyhow::Context;
use tauri::Manager;
use tracing::{
  error,
  info,
  warn
};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{
  EnvFilter,
  fmt
};

fn init_tracing() {
  let filter =
    EnvFilter::try_from_default_env()
      .or_else(|_| {
        EnvFilter::try_new(
          "info,rivet_gui_tauri=debug,\
           rivet_core=debug"
        )
      })
      .unwrap_or_else(|_| {
        EnvFilter::new("info")
      });

  let _ =
    tracing_subscriber::registry()
      .with(filter)
      .with(
        fmt::layer()
          .with_target(true)
          .with_line_number(true)
      )
      .try_init();
}

#[cfg(target_os = "linux")]
fn configure_wayland_defaults() {
  let defaults = [
    // Prefer native Wayland backend
    // for GTK/WebKit.
    ("GDK_BACKEND", "wayland"),
    // Keep winit on Wayland to avoid
    // mixed backend behavior.
    ("WINIT_UNIX_BACKEND", "wayland"),
    // Work around compositor/driver
    // dmabuf instability on some
    // systems.
    (
      "WEBKIT_DISABLE_DMABUF_RENDERER",
      "1"
    )
  ];

  for (key, value) in defaults {
    if env::var_os(key).is_none() {
      unsafe {
        env::set_var(key, value);
      }
      info!(
        key,
        value,
        "set linux GUI runtime default"
      );
    } else {
      info!(
        key,
        "preserving existing linux \
         GUI runtime value"
      );
    }
  }

  info!(
        gdk_backend = env::var("GDK_BACKEND").unwrap_or_else(|_| "<unset>".to_string()),
        winit_unix_backend =
            env::var("WINIT_UNIX_BACKEND").unwrap_or_else(|_| "<unset>".to_string()),
        webkit_disable_dmabuf_renderer =
            env::var("WEBKIT_DISABLE_DMABUF_RENDERER").unwrap_or_else(|_| "<unset>".to_string()),
        "effective linux GUI runtime backends"
    );
}

#[cfg(not(target_os = "linux"))]
fn configure_wayland_defaults() {}

fn main() {
  init_tracing();
  configure_wayland_defaults();

  info!("starting Rivet GUI backend");

  let state =
    match state::AppState::new()
      .context(
        "failed to initialize app \
         state"
      ) {
      | Ok(state) => state,
      | Err(err) => {
        error!(error = %err, "initialization failed");
        std::process::exit(1);
      }
    };

  tauri::Builder::default()
    .setup(|app| {
      configure_main_window_icon(app);
      Ok(())
    })
    .manage(state)
    .invoke_handler(
      tauri::generate_handler![
        commands::tasks_list,
        commands::task_add,
        commands::task_update,
        commands::task_done,
        commands::task_delete,
        commands::ui_log,
      ]
    )
    .run(tauri::generate_context!())
    .expect(
      "error while running Rivet GUI \
       backend"
    );
}

fn configure_main_window_icon<
  R: tauri::Runtime
>(
  app: &tauri::App<R>
) {
  let Some(window) =
    app.get_webview_window("main")
  else {
    warn!(
      "main window not found during \
       setup; skipping icon override"
    );
    return;
  };

  match tauri::image::Image::from_bytes(
    include_bytes!("../icons/icon.png")
  ) {
    | Ok(icon) => {
      if let Err(err) =
        window.set_icon(icon)
      {
        error!(error = %err, "failed to set main window icon");
      } else {
        info!(
          "set main window icon from \
           bundled app icon"
        );
      }
    }
    | Err(err) => {
      error!(error = %err, "failed to decode main window icon bytes");
    }
  }
}
