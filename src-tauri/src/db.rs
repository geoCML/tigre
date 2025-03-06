use crate::appstate::AppState;
use crate::output::Output;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use tauri::{Emitter, Manager, State};
use tokio::sync::Mutex;

#[tauri::command]
pub async fn get_as_wkt(
    table: &str,
    bb: Vec<Vec<f32>>,
    app: tauri::AppHandle,
) -> Result<Vec<String>, ()> {
    let state: State<'_, Mutex<AppState>> = app.app_handle().state();

    let _ = state.lock().await.app_handle.emit("loading", 10);

    if bb.len() != 2 {
        let _ = state.lock().await.app_handle.emit("loading", 0);
        panic!("Bounding box has fewer than 2 corners.");
    }

    let _ = state.lock().await.app_handle.emit("loading", 25);

    let mut pgsql_client =
        match Client::connect(state.lock().await.pgsql_connection.as_str(), NoTls) {
            Ok(val) => val,
            Err(_) => {
                let _ = state.lock().await.app_handle.emit("loading", 0);
                panic!("ERROR! Lost connection to the database.");
            }
        };

    let mut wkt_rows: Vec<String> = vec![];
    let wkt_result = pgsql_client.query(
        format!(
            "SELECT ST_AsText(ST_Intersection(ST_MakeEnvelope({}, {}, {}, {}), geom)) FROM {}",
            bb[0][0], bb[0][1], bb[1][0], bb[1][1], table
        )
        .as_str(),
        &[],
    );

    let _ = state.lock().await.app_handle.emit("loading", 50);
    match wkt_result {
        Ok(val) => {
            for row in val {
                for col in row.columns() {
                    wkt_rows.push(row.get::<&str, &str>(col.name()).to_string());
                }
            }
        },
        Err(_) => ()
    }

    let _ = state.lock().await.app_handle.emit("loading", 0);
    Ok(wkt_rows)
}

#[tauri::command]
pub async fn get_as_json(
    table: &str,
    bb: Vec<Vec<f32>>,
    app: tauri::AppHandle,
) -> Result<String, ()> {
    if bb.len() != 2 {
        panic!("Bounding box has fewer than 2 corners.");
    }

    let state: State<'_, Mutex<AppState>> = app.app_handle().state();
    let mut pgsql_client =
        match Client::connect(state.lock().await.pgsql_connection.as_str(), NoTls) {
            Ok(val) => val,
            Err(_) => {
                panic!("ERROR! Lost connection to the database.");
            }
        };

    let geojson_result = pgsql_client.query(
        format!("SELECT json_build_object('type', 'Feature', 'geometry', ST_AsGeoJSON(ST_Intersection(ST_MakeEnvelope({}, {}, {}, {}), geom))::json) FROM {}", bb[0][0], bb[0][1], bb[1][0], bb[1][1], table).as_str(),
        &[],
    );

    let mut json_rows: Vec<String> = vec![];
    for row in geojson_result.unwrap() {
        for col in row.columns() {
            json_rows.push(row.get::<&str, serde_json::Value>(col.name()).to_string());
        }
    }

    Ok(format!("[{}]", json_rows.join(",")))
}

async fn db_connect(
    ast: &HashMap<&str, Vec<&str>>,
    state: &State<'_, Mutex<AppState>>,
) -> Result<Output, ()> {
    let mut output = Output {
        errors: vec![],
        results: vec![],
    };

    if ast["args"].len() == 1 {
        output
            .errors
            .push("ERROR! You must provide a pgsql connection string.".to_string());
        return Ok(output);
    }

    let mut state = state.lock().await;
    let _ = &state.app_handle.emit("wipe-layers", true);

    state.pgsql_connection = ast["args"][1].to_string();
    let client = Client::connect(state.pgsql_connection.as_str(), NoTls);

    match client {
        Ok(mut val) => {
            let tables_result = &val.query("SELECT table_name, table_schema FROM information_schema.tables WHERE table_schema != 'pg_catalog' AND table_schema != 'information_schema'", &[]);
            match tables_result {
                Ok(val) => {
                    if !val.is_empty() {
                        val.iter().for_each(|row| {
                            let _ = &state.app_handle.emit("add-vector-layer", [row.get::<usize, &str>(0), row.get::<usize, &str>(1)]);
                        });
                    }
                },
                Err(_) => {
                    output.errors.push("ERROR! Failed to load layers from database.".to_string());
                }
            }
            output.results.push("Connected to database".to_string());
        }
        Err(_) => {
            output
                .errors
                .push("ERROR! Failed to connect to database.".to_string());
            state.pgsql_connection = String::new();
        }
    }

    Ok(output)
}

pub async fn db(
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
            .push("ERROR! No arguments provided for command 'db'.".to_string())
    } else {
        match ast["args"][0] {
            "connect" => {
                let db_connect_output = db_connect(ast, state).await.unwrap();
                output.errors.extend(db_connect_output.errors);
                output.results.extend(db_connect_output.results);
            }
            "current" => {
                output
                    .results
                    .extend(vec![state.lock().await.pgsql_connection.clone()]);
            }
            &_ => output
                .errors
                .push("ERROR! Found unknown argument.".to_string()),
        }
    }
    Ok(output)
}
