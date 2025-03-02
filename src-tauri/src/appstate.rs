use postgres::{Client, Error};
use tauri::AppHandle;

pub struct AppState {
    pub app_handle: AppHandle,
    pub pgsql_connection: String,
    pub pgsql_client: Result<Client, Error>,
}
