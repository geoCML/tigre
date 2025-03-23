use crate::appstate::AppState;
use crate::output::Output;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use tauri::{State, Emitter};
use tokio::sync::Mutex;

async fn set_symbology(
    ast: &HashMap<&str, Vec<&str>>,
    state: &State<'_, Mutex<AppState>>,
) -> Result<Output, ()> {
    let mut output = Output {
        errors: vec![],
        results: vec![],
    };

    let state = state.lock().await;
    let _ = state.app_handle.emit("loading", 25);

    if ast["args"].len() < 3 {
        output
            .errors
            .push("ERROR! You must provide a layer and JSON to set symbology.".to_string());
        
        let _ = state.app_handle.emit("loading", 0);
        return Ok(output);
    }

    let layer = ast["args"][1];
    let symbology_json = ast["args"][2];

    let mut client = match Client::connect(&state.pgsql_connection.pg_string(), NoTls) {
        Ok(client) => client,
        Err(_) => { 
            let _ = state.app_handle.emit("loading", 0);
            output
                .errors
                .push("ERROR! You must connect to a database before setting the symbology of a layer.".to_string());
            return Ok(output);
        }
    };

    let _ = state.app_handle.emit("loading", 90);
    match client.execute(
        format!("COMMENT ON TABLE {} IS '{}'", layer, symbology_json).as_str(),
        &[],
    ) {
        Ok(_) => {
            output.results.push("Done.".to_string());
        }
        Err(_) => {
            output
                .errors
                .push("ERROR! Failed to set symbology.".to_string());
        }
    };

    let schema_and_name  = layer.split(".").collect::<Vec<&str>>();
    if schema_and_name.len() == 2 {
        let _ = state.app_handle.emit("add-vector-layer", [schema_and_name[1].to_string(), schema_and_name[0].to_string()]);
    } else {
        let _ = state.app_handle.emit("add-raster-layer", [schema_and_name[0].to_string()]);
    }

    let _ = state.app_handle.emit("loading", 0);
    Ok(output)
}

pub async fn symbology(
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
            .push("ERROR! No arguments provided for command 'symbology'.".to_string())
    } else {
        match ast["args"][0] {
            "set" => {
                let set_symbology_output = set_symbology(ast, state).await.unwrap();
                output.errors.extend(set_symbology_output.errors);
                output.results.extend(set_symbology_output.results);
            }
            &_ => output
                .errors
                .push("ERROR! Found unknown argument.".to_string()),
        }
    }
    Ok(output)
}
