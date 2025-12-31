use flux_core::{commands, repo::branch::Branch};
use serde::Serialize;

#[derive(Serialize)]
struct BranchInfo {
    name: String,
    is_current: bool
}

#[tauri::command]
fn branches() -> Vec<BranchInfo> {
    let all_branches: Vec<Branch> = commands::get_branches().unwrap_or(Vec::new());

    all_branches
        .into_iter()
        .map(|b| BranchInfo {
            name: b.name.to_string(),
            is_current: b.is_current
        })
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![branches])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
