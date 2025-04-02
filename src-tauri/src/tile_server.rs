use std::fs;

#[tauri::command]
pub async fn map(layer: &str) -> Result<String, ()> {
    if !fs::exists(format!("/tmp/tigre/{}.svg", layer)).unwrap() {
        println!("Layer not found: {}", layer);
        return Ok(String::new());
    }

    Ok(fs::read_to_string(format!("/tmp/tigre/{}.svg", layer)).unwrap())
}