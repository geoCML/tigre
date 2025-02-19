use crate::appstate::AppState;
use crate::output::Output;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tauri::{State, Manager};


#[tauri::command]
pub async fn get_as_json(table: &str, bb: Vec<Vec<f32>>, app: tauri::AppHandle) -> Result<String, ()> {
    if bb.len() != 2 {
        panic!("Bounding box has fewer than 2 corners.");
    }

    let state: State<'_, Mutex<AppState>> = app.app_handle().state();
    let mut pgsql_client = match Client::connect(state.lock().await.pgsql_connection.as_str(), NoTls) {
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

    state.pgsql_connection = ast["args"][1].to_string();
    state.pgsql_client = Client::connect(state.pgsql_connection.as_str(), NoTls);

    match &state.pgsql_client {
        Ok(_) => {
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

pub async fn db(ast: &HashMap<&str, Vec<&str>>, state: &State<'_, Mutex<AppState>>) -> Result<Output, ()> {
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
