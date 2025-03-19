use crate::appstate::AppState;
use crate::output::Output;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use tauri::{Emitter, State};
use tokio::sync::Mutex;

pub async fn inspect(
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
            .push("ERROR! No arguments provided for command 'inspect'.".to_string())
    } else {
        let state = state.lock().await;
        let _ = state.app_handle.emit("loading", 25);
        let layer = ast["args"][0];
        let layer_split = layer.split(".").collect::<Vec<&str>>();
        let short_layer = match layer_split.len() {
            2 => layer_split[1],
            _ => layer_split[0],
        };

        let mut pgsql_client = match Client::connect(state.pgsql_connection.as_str(), NoTls) {
            Ok(val) => val,
            Err(_) => panic!("ERROR! Lost connection to the database."),
        };

        let _ = state.app_handle.emit("loading", 70);
        if ast["args"].len() == 2 {
            let location = ast["args"][1];

            match pgsql_client.query(
                format!("SELECT to_jsonb(dta) FROM (SELECT json_agg({}) FROM {} WHERE ST_Intersects(geom, ST_MakePoint({})) = TRUE) dta", short_layer, layer, location).as_str(),
                &[]
            ) {
                Ok(val) => {
                    match val.first() {
                        Some(row) => {
                            let _ = state.app_handle.emit("loading", 90);
                            let _ = state.app_handle.emit("open-table", format!("{:?}", row.get::<usize, serde_json::Value>(0).to_string()));
                            output.results.push("Done.".to_string());
                        },
                        None => output.results.push("Found 0 results.".to_string())
                    }
                },
                Err(err) => output.errors.push(format!("ERROR! Couldn't inspect layer: {}", err))
            };
        } else {
            match pgsql_client.query(
                format!(
                    "SELECT to_jsonb(dta) FROM (SELECT json_agg({}) FROM {}) dta",
                    short_layer, layer
                )
                .as_str(),
                &[],
            ) {
                Ok(val) => match val.first() {
                    Some(row) => {
                        let _ = state.app_handle.emit("loading", 90);
                        let _ = state.app_handle.emit(
                            "open-table",
                            format!("{:?}", row.get::<usize, serde_json::Value>(0).to_string()),
                        );
                        output.results.push("Done.".to_string());
                    }
                    None => output.results.push("Found 0 results.".to_string()),
                },
                Err(err) => output
                    .errors
                    .push(format!("ERROR! Couldn't inspect layer: {}", err)),
            };
        }

        let _ = state.app_handle.emit("loading", 0);
    }

    Ok(output)
}

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
            _ => layer_split[0],
        };
        let buffer_size = ast["args"][1];

        let mut pgsql_client = match Client::connect(state.pgsql_connection.as_str(), NoTls) {
            Ok(val) => val,
            Err(_) => panic!("ERROR! Lost connection to the database."),
        };

        let _ = state.app_handle.emit("loading", 70);
        match pgsql_client.execute(
            format!("CREATE TABLE IF NOT EXISTS public.{}_buffer AS SELECT ST_Buffer(geom, {}) AS geom FROM {}", short_layer, buffer_size, layer).as_str(),
            &[]
        ) {
            Ok(_) => {
                let _ = state.app_handle.emit("loading", 90);
                let _ = state.app_handle.emit("add-vector-layer", [format!("{}_buffer", short_layer), "public".to_string()]);
                output.results.push("Done.".to_string());
            },
            Err(err) => output.errors.push(format!("ERROR! Couldn't create buffer: {}", err))
        };

        let _ = state.app_handle.emit("loading", 0);
    }

    Ok(output)
}

pub async fn intersect(
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
            .push("ERROR! No arguments provided for command 'intersect'.".to_string())
    } else if ast["args"].len() != 2 {
        output
            .errors
            .push("ERROR! Not enough arguments provided for command 'intersect'.".to_string())
    } else {
        let state = state.lock().await;
        let _ = state.app_handle.emit("loading", 25);

        let layer_1 = ast["args"][0];
        let layer_1_split = layer_1.split(".").collect::<Vec<&str>>();
        let short_layer_1 = match layer_1_split.len() {
            2 => layer_1_split[1],
            _ => layer_1_split[0],
        };

        let layer_2 = ast["args"][1];
        let layer_2_split = layer_2.split(".").collect::<Vec<&str>>();
        let short_layer_2 = match layer_2_split.len() {
            2 => layer_2_split[1],
            _ => layer_2_split[0],
        };

        let mut pgsql_client = match Client::connect(state.pgsql_connection.as_str(), NoTls) {
            Ok(val) => val,
            Err(_) => panic!("ERROR! Lost connection to the database."),
        };

        let _ = state.app_handle.emit("loading", 70);
        match pgsql_client.execute(
            format!("CREATE TABLE IF NOT EXISTS public.{}_{}_intersect AS SELECT ST_Intersection({}.geom, {}.geom) AS geom FROM {}, {}", short_layer_1, short_layer_2, layer_1, layer_2, layer_1, layer_2).as_str(),
            &[]
        ) {
            Ok(_) => {
                let _ = state.app_handle.emit("loading", 90);
                let _ = state.app_handle.emit("add-vector-layer", [format!("{}_{}_intersect", short_layer_1, short_layer_2), "public".to_string()]);
                output.results.push("Done.".to_string());
            },
            Err(err) => output.errors.push(format!("ERROR! Couldn't create intersection: {}", err))
        };

        let _ = state.app_handle.emit("loading", 0);
    }

    Ok(output)
}
