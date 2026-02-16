mod commands;
mod state;

use anyhow::Context;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,rivet_gui_tauri=debug,rivet_core=debug"))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).with_line_number(true))
        .try_init();
}

fn main() {
    init_tracing();

    info!("starting Rivet GUI backend");

    let state = match state::AppState::new().context("failed to initialize app state") {
        Ok(state) => state,
        Err(err) => {
            error!(error = %err, "initialization failed");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::tasks_list,
            commands::task_add,
            commands::task_update,
            commands::task_done,
            commands::task_delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Rivet GUI backend");
}
