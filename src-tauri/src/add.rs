#![feature(file_buffered)]
use crate::appstate::AppState;
use crate::output::Output;
use crate::db::PGConnection;
use crate::gdal_utils::{generic_to_postgis_layer, generic_to_gpkg};
use postgres::{Client, NoTls};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::{Emitter, State};
use tokio::sync::Mutex;
use gdal::Dataset;
use gdal::vector::LayerAccess;

pub async fn add_layer(
    ast: &HashMap<&str, Vec<&str>>,
    state: &State<'_, Mutex<AppState>>,
) -> Result<Output, ()> {
    let mut output = Output {
        errors: vec![],
        results: vec![],
    };

    let state = state.lock().await;
    let dataset_path = ast["args"][1].to_string();

    let _ = state.app_handle.emit("loading", 10);
    if state.pgsql_connection == PGConnection::default() {
        output
            .errors
            .push("ERROR! You must connect to a database before adding a layer.".to_string());
        let _ = state.app_handle.emit("loading", 0);
        return Ok(output);
    }

    if ast["args"].len() == 1 {
        output
            .errors
            .push("ERROR! You must provide a path to the layer you want to add.".to_string());
        let _ = state.app_handle.emit("loading", 0);
        return Ok(output);
    }

    if !fs::exists(dataset_path.clone()).unwrap() {
        output
            .errors
            .push("ERROR! Path to layer does not exist on disk.".to_string());
        let _ = state.app_handle.emit("loading", 0);
        return Ok(output);
    }

    let pgsql_client = match Client::connect(&state.pgsql_connection.pg_string(), NoTls) {
        Ok(val) => val,
        Err(_) => {
            output
                .errors
                .push("ERROR! Lost connection to database.".to_string());
            let _ = state.app_handle.emit("loading", 0);
            return Ok(output);
        }
    };

    let dataset = match Dataset::open(Path::new(dataset_path.as_str())) {
        Ok(val) => val,
        Err(_) => {
            output
                .errors
                .push("ERROR! File is not a valid dataset.".to_string());
            let _ = state.app_handle.emit("loading", 0);
            return Ok(output);
        }
    };

    let mut name = match dataset.layer(0) {
        Ok(val) => val.name(),
        Err(_) => {
            output
                .errors
                .push("ERROR! Dataset has no layers.".to_string());
            let _ = state.app_handle.emit("loading", 0);
            return Ok(output);
        }
    };

    name.make_ascii_lowercase();

    let _ = fs::create_dir("/tmp/tigre");
    let _ = state.app_handle.emit("loading", 85);

    let _ = match generic_to_gpkg(dataset).await {
        Ok(_) => (),
        Err(_) => {
            output
                .errors
                .push("ERROR! Failed to write layer to gpkg.".to_string());
            let _ = state.app_handle.emit("loading", 0);
            return Ok(output);
        }
    };
    let _ = state.app_handle.emit("add-vector-layer", [name.clone(), "public".to_string()]);

    generic_to_postgis_layer(
        Dataset::open(Path::new(dataset_path.as_str())).unwrap(),  // TODO: I hate this. I want to use a reference, but I can't send a reference to a dataset between threads.
        pgsql_client,
        &name,
    )
    .await;

    output.results.push(format!("Done."));
    let _ = state.app_handle.emit("loading", 0);
    Ok(output)
}

pub async fn add(
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
            .push("ERROR! No arguments provided for command 'add'.".to_string())
    } else {
        match ast["args"][0] {
            "layer" => {
                let add_layer_output = add_layer(ast, state).await.unwrap();
                output.errors.extend(add_layer_output.errors);
                output.results.extend(add_layer_output.results);
            }
            &_ => output
                .errors
                .push("ERROR! Found unknown argument.".to_string()),
        }
    }

    Ok(output)
}
