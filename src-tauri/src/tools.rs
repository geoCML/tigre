use crate::output::Output;
use crate::appstate::AppState;
use tokio::sync::Mutex;
use postgres::{Client, NoTls};
use tauri::{Emitter, State};
use std::collections::HashMap;

pub async fn buffer(
    ast: &HashMap<&str, Vec<&str>>,
    state: &State<'_, Mutex<AppState>>,
) -> Result<Output, ()> {
    let mut output = Output {
        errors: vec![],
        results: vec![],
    };

    if ast["args"].is_empty() {
        output
            .errors
            .push("ERROR! No arguments provided for command 'buffer'.".to_string())
    } else if ast["args"].len() != 2 {
        output
            .errors
            .push("ERROR! Not enough arguments provided for command 'buffer'.".to_string())
    } else {
        let state = state.lock().await;
        let _ = state.app_handle.emit("loading", 25);
        let layer = ast["args"][0];

        let layer_split = layer.split(".").collect::<Vec<&str>>();
        let short_layer = match layer_split.len() {
            2 => layer_split[1],
            _ => layer_split[0]
        };
        let buffer_size = ast["args"][1];

        let mut pgsql_client =
            match Client::connect(state.pgsql_connection.as_str(), NoTls) {
                Ok(val) => val,
                Err(_) => panic!("ERROR! Lost connection to the database.")
            };

        let _ = state.app_handle.emit("loading", 70);
        match pgsql_client.execute(
            format!("CREATE TABLE IF NOT EXISTS {}_buffer AS SELECT ST_Buffer(geom, {}) AS geom FROM {}", layer, buffer_size, layer).as_str(),
            &[]
        ) {
            Ok(_) => {
                let _ = state.app_handle.emit("loading", 90);
                let _ = state.app_handle.emit("add-vector-layer", [format!("{}_buffer", short_layer), "public".to_string()]);
                output.results.push("Done.".to_string());
            },
            Err(_) => output.errors.push("ERROR! Couldn't create buffer.".to_string())
        };

        let _ = state.app_handle.emit("loading", 0);
    }

    Ok(output)
}
