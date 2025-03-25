use crate::db::PGConnection;
use postgres::{Client, Error};
use tauri::AppHandle;

pub struct AppState {
    pub app_handle: AppHandle,
    pub pgsql_connection: PGConnection,
    pub pgsql_client: Result<Client, Error>,
    pub hytigre: bool,
}
