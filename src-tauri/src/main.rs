#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backend;

use backend::{list_adapters, list_bt, list_wifi, run_backend_command, submit_geo};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_wifi,
            list_bt,
            list_adapters,
            submit_geo,
            run_backend_command
        ])
        .setup(|_app| {
            tauri::async_runtime::spawn(async move {
                if let Err(e) = backend::run_backend().await {
                    tracing::error!("Backend failed: {:?}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri app");
}
