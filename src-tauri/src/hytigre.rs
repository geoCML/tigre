use crate::output::Output;
use crate::appstate::AppState;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tauri::State;

pub async fn hytigre_on(
    state: &State<'_, Mutex<AppState>>,
) -> Result<Output, ()> {
    let mut output = Output {
        errors: vec![],
        results: vec![],
    };

    state.lock().await.hytigre = true;
    output.results.push("HyTigre has been turned on.".to_string());
    Ok(output)
}

pub async fn hytigre_off(
    state: &State<'_, Mutex<AppState>>,
) -> Result<Output, ()> {
    let mut output = Output {
        errors: vec![],
        results: vec![],
    };

    state.lock().await.hytigre = false;

    output.results.push("HyTigre has been turned off.".to_string());
    Ok(output)
}

pub async fn hytigre(
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
            .push("ERROR! No arguments provided for command 'hytigre'.".to_string())
    } else {
        match ast["args"][0] {
            "on" => {
                let hytigre_on_output = hytigre_on(state).await.unwrap();
                output.errors.extend(hytigre_on_output.errors);
                output.results.extend(hytigre_on_output.results);
            }
            "off" => {
                let hytigre_off_output = hytigre_off(state).await.unwrap();
                output.errors.extend(hytigre_off_output.errors);
                output.results.extend(hytigre_off_output.results);
            }
            &_ => output
                .errors
                .push("ERROR! Found unknown argument.".to_string()),
        }
    }
    Ok(output)
}
