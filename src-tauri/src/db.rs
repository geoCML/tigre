use crate::appstate::AppState;
use crate::output::Output;
use crate::gdal_utils::postgis_layer_to_gpkg;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use tauri::{Emitter, Manager, State};
use tokio::sync::Mutex;
use rusqlite::Connection;
use rusqlite::fallible_iterator::FallibleIterator;
use geozero::wkb::GpkgWkb;
use geozero::ToJson;
use std::fs;


#[tauri::command]
pub async fn get_as_json_gpkg(
    schema: &str,
    table: &str,
) -> Result<Vec<String>, ()> {
    let sqlite_connection = match Connection::open(format!("/tmp/tigre/{}.{}.gpkg", schema, table)) {
        Ok(val) => val,
        Err(_) => panic!("ERROR! Couldn't open gpkg.")
    };

    match sqlite_connection.prepare(
        format!("SELECT hex(geom) FROM {}", table).as_str()
    ) {
        Ok(mut val) => {
            return match val.query([]) {
                Ok(rows) => Ok(rows.map(|row| {
                    let wkb_data = hex::decode(row.get::<usize, String>(0)?).unwrap();
                    let wkb = GpkgWkb(wkb_data);
                    match wkb.to_json() {
                        Ok(json) => Ok(json),
                        Err(_) => Ok("[]".to_string())
                    }
                }).collect().expect("No data.")),
                Err(_) => Ok(vec![])
            };
        },
        Err(err) => panic!("ERROR! Couldn't query gpkg: {}", err)
    };
}


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
        }
        Err(_) => (),
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

    let _ = &state.app_handle.emit("loading", 10);
    let _ = &state.app_handle.emit("wipe-layers", true);

    state.pgsql_connection = ast["args"][1].to_string();
    state.gdal_pgsql_connection = "PG:dbname=geocml_db host=127.0.0.1 port=54XX user=geocml password=XXXX".to_string();

    let client = Client::connect(state.pgsql_connection.as_str(), NoTls);
    let _ = fs::create_dir("/tmp/tigre");
    let _ = &state.app_handle.emit("loading", 25);

    match client {
        Ok(mut client) => {
            let tables_result = &client.query("SELECT table_name, table_schema FROM information_schema.tables WHERE table_schema != 'pg_catalog' AND table_schema != 'information_schema' AND table_name != 'geometry_columns' AND table_name != 'geography_columns' AND table_name != 'spatial_ref_sys' AND table_name != 'raster_overviews' AND table_name != 'raster_columns'", &[]);
            match tables_result {
                Ok(tables_result) => {
                    let _ = &state.app_handle.emit("loading", 75);
                    if !tables_result.is_empty() {
                        for row in tables_result {
                            let schema = row.get::<usize, &str>(1);
                            let name = row.get::<usize, &str>(0);


                            postgis_layer_to_gpkg(name, schema, state.gdal_pgsql_connection.clone()).await;

                            let _ = &state.app_handle.emit(
                                "add-vector-layer",
                                [name, schema],
                            );
                        }
                    }
                    let _ = &state.app_handle.emit("loading", 90);
                }
                Err(_) => {
                    let _ = &state.app_handle.emit("loading", 0);
                    output
                        .errors
                        .push("ERROR! Failed to load layers from database.".to_string());
                }
            }
            output.results.push("Connected to database".to_string());
        }
        Err(_) => {
            output
                .errors
                .push("ERROR! Failed to connect to database.".to_string());
            state.pgsql_connection = String::new();
            let _ = &state.app_handle.emit("loading", 0);
        }
    }

    let _ = &state.app_handle.emit("loading", 0);
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
