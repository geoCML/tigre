use crate::add::add;
use crate::appstate::AppState;
use crate::db::db;
use crate::output::Output;
use crate::tools::buffer;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::string::String;
use tauri::{Manager, State};
use tokio::sync::Mutex;

#[tauri::command]
pub fn read(cmd: &str) -> HashMap<&str, Vec<String>> {
    // Setup ast and parse tokens
    let mut ast: HashMap<&str, Vec<String>> = HashMap::new();
    let mut tokens: Vec<String> = cmd
        .split_whitespace()
        .collect::<Vec<&str>>()
        .iter()
        .map(|tkn| tkn.to_string())
        .collect();

    let left_anchor: usize = tokens
        .iter()
        .position(|elem| elem.starts_with("\'"))
        .unwrap_or(0);

    let right_anchor: usize = tokens
        .iter()
        .position(|elem| elem.ends_with("\'"))
        .unwrap_or(0);

    if left_anchor + right_anchor > 0 {
        let mut string_token: String = String::new();
        string_token =
            Vec::from_iter(tokens[left_anchor..right_anchor + 1].iter().cloned()).join(" ");
        string_token.remove(0);
        string_token.remove(string_token.len() - 1);

        let mut replacement_vector: Vec<String> =
            vec![String::new(); (right_anchor - left_anchor) + 1];
        replacement_vector[0] = string_token;
        tokens.splice(left_anchor..right_anchor + 1, replacement_vector);
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

pub async fn foo() {}

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
        },
        "buffer" => {
            let buffer_output = buffer(&ast, &state).await.unwrap();
            output.errors.extend(buffer_output.errors);
            output.results.extend(buffer_output.results);
        },
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
