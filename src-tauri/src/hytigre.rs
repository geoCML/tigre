use crate::output::Output;
use crate::appstate::AppState;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tauri::State;
use actix_web::{App, HttpServer, Responder, HttpResponse};
use tauri::async_runtime::spawn;
use actix_web::web;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("If you're reading this, HyTigre is active and the server is running!")
}

pub async fn start_server() -> std::io::Result<actix_web::dev::Server> {
    let server = HttpServer::new(|| App::new()
            .route("/", web::get().to(index))
        )
        .bind(("127.0.0.1", 8080))?
        .run();

    Ok(server) 
}

pub async fn hytigre_on(
    state: &State<'_, Mutex<AppState>>,
) -> Result<Output, ()> {
    let mut output = Output {
        errors: vec![],
        results: vec![],
    };

    state.lock().await.hytigre = Some(spawn(async move { 
        println!("Server started on port 8080");
        start_server().await.unwrap().await
    }));
 
    output.results.push("HyTigre has been turned on.".to_string());
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
            },
            &_ => output
                .errors
                .push("ERROR! Found unknown argument.".to_string()),
        }
    }
    Ok(output)
}
