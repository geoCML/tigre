#![feature(file_buffered)]
use crate::appstate::AppState;
use crate::output::Output;
use gdal::vector::{Geometry, LayerAccess};
use gdal::Dataset;
use gdal_sys::GDALVectorTranslate;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::ptr::{null, null_mut};
use std::string::String;
use tauri::{Emitter, State};
use tokio::sync::Mutex;

pub async fn add_layer(
    ast: &HashMap<&str, Vec<&str>>,
    state: &State<'_, Mutex<AppState>>,
) -> Result<Output, ()> {
    let mut output = Output {
        errors: vec![],
        results: vec![],
    };

    let state = state.lock().await;

    let _ = state.app_handle.emit("loading", 10);
    if state.pgsql_connection == String::new() {
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

    if !fs::exists(ast["args"][1]).unwrap() {
        output
            .errors
            .push("ERROR! Path to layer does not exist on disk.".to_string());
        let _ = state.app_handle.emit("loading", 0);
        return Ok(output);
    }

    let mut pgsql_client = match Client::connect(state.pgsql_connection.as_str(), NoTls) {
        Ok(val) => val,
        Err(_) => {
            output
                .errors
                .push("ERROR! Lost connection to database.".to_string());
            let _ = state.app_handle.emit("loading", 0);
            return Ok(output);
        }
    };

    let file = match Dataset::open(Path::new(ast["args"][1])) {
        Ok(val) => val,
        Err(_) => {
            output
                .errors
                .push("ERROR! File is not a valid dataset.".to_string());
            let _ = state.app_handle.emit("loading", 0);
            return Ok(output);
        }
    };

    let mut name = match file.layer(0) {
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

    let mut fields: Vec<String> = vec![];
    let mut geometries: Vec<Geometry> = vec![];
    let mut geometry_type = String::new();

    let _ = state.app_handle.emit("loading", 25);
    unsafe {
        let mut raw_dataset: Vec<gdal_sys::GDALDatasetH> = vec![file.c_dataset()];

        GDALVectorTranslate(
            //null(),
            //CString::new(state.lock().unwrap().pgsql_connection.clone()).unwrap().as_ptr(),
            CString::new(format!("/tmp/{}.csv", name)).unwrap().as_ptr(),
            null_mut(),
            1,
            raw_dataset.as_mut_ptr(),
            null(),
            null_mut(),
        );
    };

    let _ = state.app_handle.emit("loading", 50);
    file.layers()
        .for_each(| mut lyr | {
            // COLLECT LAYER GEOMETRIES
            lyr.features()
                .for_each(| feature | {
                    match feature.geometry() {
                        Some(geometry) => geometries.push(geometry.clone()),
                        _ => ()
                    }
                });

            // COLLECT FIELD TYPES
            for geometry in geometries.clone() {
                if geometry_type == "" {
                    geometry_type = geometry.geometry_name();
                    continue;
                }

                if geometry_type != geometry.geometry_name() {
                    output.errors.push(format!("ERROR! Some features in layer have mismatched geometries. Expected {}, but got {}.", geometry_type, geometry.geometry_name()));
                    break;
                }
            }

            lyr.defn()
                .fields()
                .for_each(| f | {
                    let pg_field_type = match f.field_type() {
                        8 => "bytea",
                        9 => "date",
                        11 => "timestamp",
                        0 => "integer",
                        12 => "bigint",
                        13 => "bigint[]",
                        1 => "integer[]",
                        2 => "numeric",
                        3 => "numeric[]",
                        4 => "text",
                        5 => "text[]",
                        10 => "time",
                        6 => "text",
                        7 => "text[]",
                        _ => "text"
                    };
                    fields.push(format!("{} {}", f.name(), pg_field_type));
                });
        });

    if output.errors.len() > 0 {
        let _ = state.app_handle.emit("loading", 0);
        return Ok(output);
    }

    let _ = state.app_handle.emit("loading", 60);
    // CREATE TABLE
    let create_layer_result = pgsql_client.execute(
        format!(
            "CREATE TABLE {} ({}, geom geometry)",
            name,
            fields.join(", ")
        )
        .as_str(),
        &[],
    );
    match create_layer_result {
        Ok(_) => (),
        Err(ref err) => {
            output
                .errors
                .push("ERROR! Failed to create layer in database.".to_string());
            output.errors.push(err.to_string());

            let _ = state.app_handle.emit("loading", 0);
            let _ = fs::remove_file(format!("/tmp/{}.csv", &name));
            return Ok(output);
        }
    };

    // SET GEOMETRY TYPE
    let set_geometry_query = match geometries[0].spatial_ref() {
        Some(val) => {
            let srid = match val.auth_code() {
                Ok(val) => val,
                Err(_) => 4326
            };

            format!(
                "ALTER TABLE \"{}\" ALTER COLUMN geom TYPE Geometry({}, {})",
                name,
                geometry_type,
                srid
            )
        },
        None => format!(
            "ALTER TABLE \"{}\" ALTER COLUMN geom TYPE Geometry({})",
            name, geometry_type
        ),
    };

    let set_geometry_result = pgsql_client.execute(set_geometry_query.as_str(), &[]);
    match set_geometry_result {
        Ok(_) => (),
        Err(ref err) => {
            output
                .errors
                .push("ERROR! Failed to set geometry information.".to_string());
            output.errors.push(err.to_string());
            return Ok(output);
        }
    };

    let _ = state.app_handle.emit("loading", 90);
    // COPY FROM CSV -> NEW PGSQL TABLE
    let csv_file = File::open_buffered(format!("/tmp/{}.csv", name).as_str()).unwrap();
    let mut cols = String::new();
    let mut queries: Vec<String> = vec![];

    for (line, i) in csv_file.lines().zip(1..) {
        if i == 1 {
            cols = line
                .unwrap()
                .clone()
                .as_str()
                .split(",")
                .collect::<Vec<_>>()
                .join(", ");

            continue;
        }
        let the_line = line
            .unwrap()
            .clone()
            .as_str()
            .split(",")
            .collect::<Vec<_>>()
            .into_iter()
            .map(|item| {
                return format!("\'{}\'", item);
            })
            .collect::<Vec<_>>()
            .join(", ");

        queries.push(format!(
            "INSERT INTO {} ({}, geom) VALUES ({}, '{}')",
            name,
            cols,
            the_line,
            geometries[i - 2].wkt().unwrap()
        ))
    }

    let insert_result = pgsql_client.batch_execute(queries.join(";").as_str());
    match insert_result {
        Ok(_) => (),
        Err(ref err) => {
            output
                .errors
                .push("ERROR! Failed to load raw data into database table.".to_string());
            output.errors.push(err.to_string());
            let _ = state.app_handle.emit("loading", 0);
            return Ok(output);
        }
    }

    let _ = fs::remove_file(format!("/tmp/{}.csv", name));
    let _ = state.app_handle.emit("add-vector-layer", [name, "public".to_string()]);
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
