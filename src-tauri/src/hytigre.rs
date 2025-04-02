use crate::output::Output;
use crate::appstate::AppState;
use crate::db::{get_as_json, get_layer_symbology, inspect_layer, inspect_layer_at_location};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tauri::{State, Manager};
use actix_web::{App, HttpServer, Responder, HttpResponse};
use tauri::async_runtime::spawn;
use actix_web::web;

#[derive(serde::Serialize)]
struct Response {
    message: String,
    result: Option<String>
}

async fn index() -> impl Responder {
    HttpResponse::Ok().json(Response {
        message: "Congratulations! If you're reading this, HyTigre is active and the server is running!".to_string(),
        result: None
    })
}

#[derive(serde::Deserialize)]
struct GeometryRequest {
    table: String,
    bb: Vec<Vec<f32>>
}

async fn geometry(req: web::Json<GeometryRequest>, app_handle: web::Data<tauri::AppHandle>) -> impl Responder {
    let res: String = match get_as_json(&req.table, req.bb.clone(), app_handle.get_ref().clone()).await {
        Ok(val) => val,
        Err(e) => {
            return HttpResponse::BadRequest().json(Response {
                message: e,
                result: None
            });
        }
    };

    HttpResponse::Ok().json(Response {
        message: "Done.".to_string(),
        result: Some(res)
    })
}

#[derive(serde::Deserialize)]
struct SymbologyRequest {
    schema: String,
    table: String
}

async fn symbology(req: web::Json<SymbologyRequest>, app_handle: web::Data<tauri::AppHandle>) -> impl Responder {
    let res: String = match get_layer_symbology(&req.schema, &req.table, app_handle.get_ref().clone()).await {
        Ok(val) => val,
        Err(e) => {
            return HttpResponse::BadRequest().json(Response {
                message: e,
                result: None
            });
        }
    };

    HttpResponse::Ok().json(Response {
        message: "Done.".to_string(),
        result: Some(res)
    })
}

#[derive(serde::Deserialize)]
struct InspectRequest {
    table: String
}

async fn inspect(req: web::Json<InspectRequest>, app_handle: web::Data<tauri::AppHandle>) -> impl Responder {
    let state: State<'_, Mutex<AppState>> = app_handle.get_ref().state();
    let res: String = match inspect_layer(&req.table, &state.lock().await.pgsql_connection).await {
        Ok(val) => val,
        Err(e) => {
            return HttpResponse::BadRequest().json(Response {
                message: e,
                result: None
            });
        }
    };

    HttpResponse::Ok().json(Response {
        message: "Done.".to_string(),
        result: Some(res)
    })
}

#[derive(serde::Deserialize)]
struct InspectAtLocationRequest {
    table: String,
    location: String
}

async fn inspect_location(req: web::Json<InspectAtLocationRequest>, app_handle: web::Data<tauri::AppHandle>) -> impl Responder {
    let state: State<'_, Mutex<AppState>> = app_handle.get_ref().state();
    let res: String = match inspect_layer_at_location(&req.table, &state.lock().await.pgsql_connection, &req.location).await {
        Ok(val) => val,
        Err(e) => {
            return HttpResponse::BadRequest().json(Response {
                message: e,
                result: None
            });
        }
    };

    HttpResponse::Ok().json(Response {
        message: "Done.".to_string(),
        result: Some(res)
    })
}


pub async fn start_server(app_handle: tauri::AppHandle) -> std::io::Result<actix_web::dev::Server> {
    let server = HttpServer::new(move || App::new()
            .app_data(web::Data::new(app_handle.app_handle().clone()))
            .route("/", web::get().to(index))
            .route("/geometry", web::get().to(geometry))
            .route("/symbology", web::get().to(symbology))
            .route("/inspect", web::get().to(inspect))
            .route("/inspect-location", web::get().to(inspect_location))
        )
        .bind(("127.0.0.1", 3000))?
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

    let mut state = state.lock().await;
    let app_handle = state.app_handle.clone();
    match &state.hytigre {
        Some(handle) => {
            handle.abort();
        }
        None => {}
    };

    state.hytigre = Some(spawn(async move { 
        start_server(app_handle).await.unwrap().await
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
