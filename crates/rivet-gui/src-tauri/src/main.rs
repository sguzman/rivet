mod commands;
mod state;

use std::path::PathBuf;
use std::{
  env,
  fs
};

use anyhow::Context;
use chrono::Local;
use serde::Deserialize;
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

const APP_CONFIG_FILE: &str =
  "rivet.toml";
const APP_CONFIG_ENV_VAR: &str =
  "RIVET_CONFIG";
const LEGACY_APP_CONFIG_ENV_VAR: &str =
  "RIVET_APP_CONFIG";
const LOG_DIR_NAME: &str = "logs";
const LOG_FILE_PREFIX: &str =
  "rivet-gui";

#[derive(
  Debug, Clone, Copy, PartialEq, Eq,
)]
enum RuntimeMode {
  Dev,
  Prod
}

impl RuntimeMode {
  fn as_str(self) -> &'static str {
    match self {
      | Self::Dev => "dev",
      | Self::Prod => "prod"
    }
  }
}

#[derive(Debug, Deserialize)]
struct RuntimeConfig {
  mode:    Option<String>,
  app:     Option<RuntimeAppConfig>,
  logging: Option<RuntimeLoggingConfig>
}

#[derive(Debug, Deserialize)]
struct RuntimeAppConfig {
  mode: Option<String>
}

#[derive(Debug, Deserialize)]
struct RuntimeLoggingConfig {
  directory:   Option<String>,
  file_prefix: Option<String>
}

#[derive(Debug, Clone)]
struct RuntimeSettings {
  mode:            RuntimeMode,
  log_directory:   String,
  log_file_prefix: String
}

fn env_is_truthy(key: &str) -> bool {
  let Some(value) = env::var_os(key)
  else {
    return false;
  };
  let text = value
    .to_string_lossy()
    .trim()
    .to_ascii_lowercase();
  matches!(
    text.as_str(),
    "1" | "true" | "yes" | "on"
  )
}

fn should_force_dev_mode() -> bool {
  // `cargo tauri dev` sets this.
  if env_is_truthy("TAURI_ENV_DEBUG") {
    return true;
  }

  // When running debug binaries
  // directly, default to dev logging.
  cfg!(debug_assertions)
}

fn find_runtime_config_upwards()
-> Option<PathBuf> {
  let cwd = env::current_dir().ok()?;
  let mut cursor = Some(cwd.as_path());
  while let Some(path) = cursor {
    let candidate =
      path.join(APP_CONFIG_FILE);
    if candidate.is_file() {
      return Some(candidate);
    }
    cursor = path.parent();
  }
  None
}

fn resolve_log_directory(
  config_path: Option<&PathBuf>,
  directory: &str
) -> String {
  let raw = PathBuf::from(directory);
  if raw.is_absolute() {
    return directory.to_string();
  }

  if let Some(path) = config_path
    && let Some(parent) = path.parent()
  {
    return parent
      .join(raw)
      .to_string_lossy()
      .to_string();
  }

  directory.to_string()
}

fn load_runtime_settings()
-> RuntimeSettings {
  let force_dev =
    should_force_dev_mode();
  let defaults = RuntimeSettings {
    mode:            if force_dev {
      RuntimeMode::Dev
    } else {
      RuntimeMode::Prod
    },
    log_directory:   LOG_DIR_NAME
      .to_string(),
    log_file_prefix: LOG_FILE_PREFIX
      .to_string()
  };

  let config_path =
    env::var(APP_CONFIG_ENV_VAR)
      .ok()
      .map(PathBuf::from)
      .or_else(|| {
        env::var(
          LEGACY_APP_CONFIG_ENV_VAR
        )
        .ok()
        .map(PathBuf::from)
      })
      .or_else(|| {
        find_runtime_config_upwards()
      });

  let Some(path) = config_path else {
    warn!(
      "runtime config path not \
       available; defaulting mode to \
       prod"
    );
    return defaults;
  };

  if !path.exists() {
    warn!(
      path = %path.display(),
      "runtime config missing; \
       defaulting mode to prod"
    );
    return defaults;
  }

  let raw =
    match fs::read_to_string(&path) {
      | Ok(raw) => raw,
      | Err(error) => {
        error!(
          path = %path.display(),
          %error,
          "failed to read runtime \
           config; defaulting mode to \
           prod"
        );
        return defaults;
      }
    };

  let parsed = match toml::from_str::<
    RuntimeConfig
  >(&raw)
  {
    | Ok(parsed) => parsed,
    | Err(error) => {
      error!(
        path = %path.display(),
        %error,
        "failed to parse runtime \
         config; defaulting mode to \
         prod"
      );
      return defaults;
    }
  };

  let mode_raw = parsed
    .app
    .as_ref()
    .and_then(|app| app.mode.as_deref())
    .or(parsed.mode.as_deref())
    .map(str::trim);
  let mode = match mode_raw {
    | Some(mode)
      if mode.eq_ignore_ascii_case(
        "dev"
      ) =>
    {
      RuntimeMode::Dev
    }
    | Some(mode)
      if mode.eq_ignore_ascii_case(
        "prod"
      ) =>
    {
      RuntimeMode::Prod
    }
    | Some(mode) => {
      warn!(
        path = %path.display(),
        mode,
        "unsupported runtime mode; \
         expected dev|prod, \
         defaulting to prod"
      );
      RuntimeMode::Prod
    }
    | None => {
      warn!(
        path = %path.display(),
        "runtime config missing mode; \
         defaulting to prod"
      );
      RuntimeMode::Prod
    }
  };

  let configured_log_directory = parsed
    .logging
    .as_ref()
    .and_then(|logging| {
      logging.directory.as_deref()
    })
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .unwrap_or(LOG_DIR_NAME)
    .to_string();
  let log_directory =
    resolve_log_directory(
      Some(&path),
      &configured_log_directory
    );
  let log_file_prefix = parsed
    .logging
    .as_ref()
    .and_then(|logging| {
      logging.file_prefix.as_deref()
    })
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .unwrap_or(LOG_FILE_PREFIX)
    .to_string();

  let mut settings = RuntimeSettings {
    mode,
    log_directory,
    log_file_prefix
  };

  if force_dev {
    settings.mode = RuntimeMode::Dev;
  }

  settings
}

fn init_tracing(
  settings: &RuntimeSettings
) {
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

  if settings.mode == RuntimeMode::Dev {
    let log_dir = env::current_dir()
      .unwrap_or_else(|_| {
        PathBuf::from(".")
      })
      .join(
        settings.log_directory.trim()
      );

    if let Err(error) =
      fs::create_dir_all(&log_dir)
    {
      let _ =
        tracing_subscriber::registry()
          .with(filter)
          .with(
            fmt::layer()
              .with_target(true)
              .with_line_number(true)
          )
          .try_init();
      error!(
        path = %log_dir.display(),
        %error,
        "failed creating log \
         directory; falling back to \
         stderr logging"
      );
      return;
    }

    let file_name = format!(
      "{}-{}.log",
      settings.log_file_prefix.trim(),
      Local::now()
        .format("%Y%m%d-%H%M%S")
    );
    let log_path =
      log_dir.join(file_name);
    let file =
      match fs::File::create(&log_path)
      {
        | Ok(file) => file,
        | Err(error) => {
          let _ =
          tracing_subscriber::registry()
            .with(filter)
            .with(
              fmt::layer()
                .with_target(true)
                .with_line_number(true)
            )
            .try_init();
          error!(
            path = %log_path.display(),
            %error,
            "failed opening log file; \
             falling back to stderr \
             logging"
          );
          return;
        }
      };

    let (non_blocking, guard) =
      tracing_appender::non_blocking(
        file
      );
    let _ =
      tracing_subscriber::registry()
        .with(filter)
        .with(
          fmt::layer()
            .with_target(true)
            .with_line_number(true)
            .with_ansi(false)
            .with_writer(non_blocking)
        )
        .try_init();
    std::mem::forget(guard);
    return;
  }

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
  let dev_forced =
    should_force_dev_mode();
  let runtime_settings =
    load_runtime_settings();
  init_tracing(&runtime_settings);
  configure_wayland_defaults();
  if dev_forced {
    info!(
      "forcing runtime mode=dev \
       (TAURI_ENV_DEBUG/debug build)"
    );
  }
  info!(
    mode =
      runtime_settings.mode.as_str(),
    log_directory = runtime_settings
      .log_directory
      .as_str(),
    log_file_prefix = runtime_settings
      .log_file_prefix
      .as_str(),
    "loaded runtime settings"
  );

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
        commands::config_snapshot,
        commands::config_apply_updates,
        commands::tag_schema_snapshot,
        commands::tasks_list,
        commands::task_add,
        commands::task_update,
        commands::task_done,
        commands::task_uncomplete,
        commands::task_delete,
        commands::contacts_list,
        commands::contact_add,
        commands::contact_update,
        commands::contact_delete,
        commands::contacts_delete_bulk,
        commands::contacts_dedupe_preview,
        commands::contacts_dedupe_candidates,
        commands::contacts_dedupe_decide,
        commands::contact_open_action,
        commands::contacts_import_preview,
        commands::contacts_import_commit,
        commands::contacts_merge,
        commands::contacts_merge_undo,
        commands::ui_log,
        commands::external_calendar_sync,
        commands::external_calendar_import_ics,
        commands::external_calendar_cache_list,
        commands::external_calendar_import_cached,
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
    "[Desktop Entry]\\
     nType=Application\nName=Rivet\\
     nComment=Rivet Taskwarrior \
     GUI\nExec={exec}\nIcon={icon}\\
     nTerminal=false\\
     nStartupNotify=true\\
     nStartupWMClass={app_id}\\
     nX-GNOME-WMClass={app_id}\\
     nCategories=Utility;\n",
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
