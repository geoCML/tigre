use crate::add::add;
use crate::appstate::AppState;
use crate::db::db;
use crate::hytigre::hytigre;
use crate::output::Output;
use crate::symbology::symbology;
use crate::tools::{buffer, inspect, intersect};
use std::collections::HashMap;
use std::string::String;
use tauri::{Manager, State};
use tokio::sync::Mutex;

#[tauri::command]
pub fn read(cmd: &str) -> HashMap<&str, Vec<String>> {
    let mut ast: HashMap<&str, Vec<String>> = HashMap::new();
    let mut tokens: Vec<String> = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;

    for c in cmd.chars() {
        match c {
            '`' => {
                if in_quotes {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                in_quotes = !in_quotes;
            }
            ' ' if !in_quotes => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            }
            _ => current_token.push(c),
        }
    }
    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    // Insert command into ast
    ast.insert("cmd", vec![tokens[0].clone()]);

    // Collect required arguments
    let mut optional_args_index = 0;
    let mut args: Vec<String> = vec![];
    for i in 1..tokens.len() {
        if tokens[i].is_empty() {
            continue;
        }

        if tokens[i] == "?" {
            optional_args_index = i;
            break;
        }
        args.push(tokens[i].clone());
    }
    ast.insert("args", args);

    // Collect optional arguments
    let mut optional_args: Vec<String> = vec![];
    for i in optional_args_index..tokens.len() {
        if tokens[i] == "?" {
            continue;
        }
        optional_args.push(tokens[i].clone());
    }
    ast.insert("optional_args", optional_args);

    ast
}

#[tauri::command]
pub async fn eval(ast: HashMap<&str, Vec<&str>>, app: tauri::AppHandle) -> Result<String, ()> {
    let state: State<'_, Mutex<AppState>> = app.app_handle().state();
    let mut output = Output {
        errors: vec![],
        results: vec![],
    };

    match ast["cmd"][0] {
        "add" => {
            let add_output = add(&ast, &state).await.unwrap();
            output.errors.extend(add_output.errors);
            output.results.extend(add_output.results);
        }
        "db" => {
            let db_output = db(&ast, &state).await.unwrap();
            output.errors.extend(db_output.errors);
            output.results.extend(db_output.results);
        }
        "buffer" => {
            let buffer_output = buffer(&ast, &state).await.unwrap();
            output.errors.extend(buffer_output.errors);
            output.results.extend(buffer_output.results);
        }
        "intersect" => {
            let intersect_output = intersect(&ast, &state).await.unwrap();
            output.errors.extend(intersect_output.errors);
            output.results.extend(intersect_output.results);
        }
        "inspect" => {
            let inspect_output = inspect(&ast, &state).await.unwrap();
            output.errors.extend(inspect_output.errors);
            output.results.extend(inspect_output.results);
        },
        "symbology" => {
            let symbology_output = symbology(&ast, &state).await.unwrap();
            output.errors.extend(symbology_output.errors);
            output.results.extend(symbology_output.results);
        },
        "hytigre" => {
            let hytigre_output = hytigre(&ast, &state).await.unwrap();
            output.errors.extend(hytigre_output.errors);
            output.results.extend(hytigre_output.results);
        }
        "save" => println!("save"),
        &_ => {
            output.errors.push("ERROR! Unknown command.".to_string());
        }
    }

    Ok(format!(
        "{{ \"errors\": {:?}, \"results\": {:?} }}",
        output.errors, output.results
    ))
}
