#![feature(file_buffered)]
pub mod add;
pub mod appstate;
pub mod db;
pub mod output;
pub mod repl;
pub mod tools;

use crate::appstate::AppState;
use crate::db::{get_as_json, get_as_wkt};
use crate::repl::{eval, read};
use postgres::{Client, NoTls};
use std::string::String;
use tauri::Manager;
use tokio::sync::Mutex;

// TAURI STUFF
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = Mutex::new(AppState {
                app_handle: app.handle().clone(),
                pgsql_connection: String::new(),
                pgsql_client: Client::connect("", NoTls),
            });

            app.manage(state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            read,
            eval,
            get_as_json,
            get_as_wkt
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
