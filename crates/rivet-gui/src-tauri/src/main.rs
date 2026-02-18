mod commands;
mod state;

use std::{
  env,
  fs
};

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
      ensure_linux_taskbar_icon_registration(
        app
      );
      install_signal_handlers(
        app.handle().clone()
      );
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
        commands::external_calendar_sync,
        commands::external_calendar_import_ics,
        commands::window_minimize,
        commands::window_toggle_maximize,
        commands::window_close,
      ]
    )
    .run(tauri::generate_context!())
    .expect(
      "error while running Rivet GUI \
       backend"
    );
}

fn install_signal_handlers(
  app_handle: tauri::AppHandle
) {
  tauri::async_runtime::spawn(
    async move {
      wait_for_shutdown_signal().await;
      warn!(
        "received shutdown signal; \
         exiting application"
      );
      app_handle.exit(0);
    }
  );
}

#[cfg(unix)]
async fn wait_for_shutdown_signal() {
  use tokio::signal::unix::{
    SignalKind,
    signal
  };

  let mut sigint = match signal(
    SignalKind::interrupt()
  ) {
    | Ok(stream) => stream,
    | Err(error) => {
      error!(
        %error,
        "failed to register SIGINT \
         handler; falling back to \
         ctrl_c"
      );
      let _ =
        tokio::signal::ctrl_c().await;
      return;
    }
  };

  let mut sigterm = match signal(
    SignalKind::terminate()
  ) {
    | Ok(stream) => stream,
    | Err(error) => {
      error!(
        %error,
        "failed to register SIGTERM \
         handler; falling back to \
         ctrl_c"
      );
      let _ =
        tokio::signal::ctrl_c().await;
      return;
    }
  };

  tokio::select! {
    _ = sigint.recv() => {}
    _ = sigterm.recv() => {}
  }
}

#[cfg(not(unix))]
async fn wait_for_shutdown_signal() {
  if let Err(error) =
    tokio::signal::ctrl_c().await
  {
    error!(
      %error,
      "failed waiting for ctrl_c \
       signal"
    );
  }
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

  let candidates: [(&str, &[u8]); 3] = [
    (
      "mascot-square",
      &include_bytes!(
        "../icons/mascot-square.png"
      )[..]
    ),
    (
      "icon",
      &include_bytes!(
        "../icons/icon.png"
      )[..]
    ),
    (
      "favicon-32x32",
      &include_bytes!(
        "../../ui/assets/icons/\
         favicon-32x32.png"
      )[..]
    )
  ];

  for (name, bytes) in candidates {
    match tauri::image::Image::from_bytes(
      bytes
    ) {
      | Ok(icon) => {
        match window.set_icon(icon) {
          | Ok(()) => {
            info!(
              icon = name,
              "set main window icon"
            );
            return;
          }
          | Err(err) => {
            error!(
              icon = name,
              error = %err,
              "failed to apply main \
               window icon candidate"
            );
          }
        }
      }
      | Err(err) => {
        error!(
          icon = name,
          error = %err,
          "failed to decode window \
           icon candidate"
        );
      }
    }
  }

  if let Some(default_icon) =
    app.default_window_icon().cloned()
  {
    match window.set_icon(default_icon)
    {
      | Ok(()) => {
        info!(
          "set main window icon from \
           tauri default window icon"
        );
        return;
      }
      | Err(err) => {
        error!(
          error = %err,
          "failed applying tauri \
           default window icon"
        );
      }
    }
  }

  warn!(
    "unable to set main window icon \
     from available icon candidates"
  );
}

#[cfg(target_os = "linux")]
fn ensure_linux_taskbar_icon_registration<
  R: tauri::Runtime
>(
  app: &tauri::App<R>
) {
  let Some(home_dir) =
    env::var_os("HOME")
  else {
    warn!(
      "HOME is unset; skipping \
       taskbar icon registration"
    );
    return;
  };

  let app_id = app
    .config()
    .identifier
    .trim()
    .to_string();
  if app_id.is_empty() {
    warn!(
      "empty app identifier; skipping \
       taskbar icon registration"
    );
    return;
  }

  let applications_dir =
    std::path::Path::new(&home_dir)
      .join(
        ".local/share/applications"
      );
  let icons_dir =
    std::path::Path::new(&home_dir)
      .join(
        ".local/share/icons/hicolor/\
         256x256/apps"
      );

  if let Err(error) = fs::create_dir_all(
    &applications_dir
  ) {
    error!(
      %error,
      path = %applications_dir.display(),
      "failed creating desktop entry \
       directory for taskbar icon \
       registration"
    );
    return;
  }
  if let Err(error) =
    fs::create_dir_all(&icons_dir)
  {
    error!(
      %error,
      path = %icons_dir.display(),
      "failed creating icon \
       directory for taskbar icon \
       registration"
    );
    return;
  }

  let icon_path = icons_dir
    .join(format!("{app_id}.png"));
  if let Err(error) = fs::write(
    &icon_path,
    include_bytes!(
      "../icons/mascot-square.png"
    )
  ) {
    error!(
      %error,
      path = %icon_path.display(),
      "failed writing taskbar icon \
       asset"
    );
    return;
  }

  let exec = env::current_exe()
    .ok()
    .and_then(|path| {
      path
        .as_os_str()
        .to_str()
        .map(|raw| raw.to_string())
    })
    .unwrap_or_else(|| {
      "rivet".to_string()
    });
  let desktop_path = applications_dir
    .join(format!("{app_id}.desktop"));
  let desktop_contents = format!(
    "[Desktop Entry]\n\
     Type=Application\n\
     Name=Rivet\n\
     Comment=Rivet Taskwarrior GUI\n\
     Exec={exec}\n\
     Icon={icon}\n\
     Terminal=false\n\
     StartupNotify=true\n\
     StartupWMClass={app_id}\n\
     X-GNOME-WMClass={app_id}\n\
     Categories=Utility;\n",
    exec = exec,
    icon = icon_path.display(),
    app_id = app_id
  );

  if let Err(error) = fs::write(
    &desktop_path,
    desktop_contents.as_bytes()
  ) {
    error!(
      %error,
      path = %desktop_path.display(),
      "failed writing desktop entry \
       for taskbar icon \
       registration"
    );
    return;
  }

  info!(
    app_id = %app_id,
    desktop_file =
      %desktop_path.display(),
    icon_file =
      %icon_path.display(),
    "registered linux desktop \
     metadata for taskbar icon \
     resolution"
  );
}

#[cfg(not(target_os = "linux"))]
fn ensure_linux_taskbar_icon_registration<
  R: tauri::Runtime
>(
  _app: &tauri::App<R>
) {
}
