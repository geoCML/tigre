#![feature(file_buffered)]
pub mod add;
pub mod appstate;
pub mod db;
pub mod output;
pub mod repl;
pub mod tools;
pub mod gdal_utils;
pub mod symbology;
pub mod hytigre;

use crate::appstate::AppState;
use crate::db::{get_as_json, get_as_wkt, get_as_json_gpkg, get_layer_symbology, PGConnection};
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
                pgsql_connection: PGConnection::default(),
                pgsql_client: Client::connect("", NoTls),
                hytigre: false,
            });

            app.manage(state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            read,
            eval,
            get_as_json,
            get_as_wkt,
            get_as_json_gpkg,
            get_layer_symbology
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
