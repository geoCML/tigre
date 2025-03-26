use crate::db::PGConnection;
use postgres::{Client, Error};
use tauri::AppHandle;
use tauri::async_runtime::JoinHandle;

pub struct AppState {
    pub app_handle: AppHandle,
    pub pgsql_connection: PGConnection,
    pub pgsql_client: Result<Client, Error>,
    pub hytigre: Option<JoinHandle<Result<(), std::io::Error>>>,
}
