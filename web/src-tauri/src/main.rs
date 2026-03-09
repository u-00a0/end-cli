#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use end_web::{Lang, bootstrap, solve_from_aic_toml};

fn parse_lang(tag: &str) -> Result<Lang, String> {
    match tag.trim().to_ascii_lowercase().as_str() {
        "zh" => Ok(Lang::Zh),
        "en" => Ok(Lang::En),
        other => Err(format!("Unknown lang `{other}` (expected `zh` or `en`)")),
    }
}

#[tauri::command]
fn cmd_bootstrap(lang: String) -> Result<serde_json::Value, String> {
    let lang = parse_lang(&lang)?;
    let payload = bootstrap(lang).map_err(|e| e.to_string())?;
    serde_json::to_value(&payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn cmd_solve(lang: String, aic_toml: String) -> Result<serde_json::Value, String> {
    let lang = parse_lang(&lang)?;
    let payload = solve_from_aic_toml(lang, &aic_toml).map_err(|e| e.to_string())?;
    serde_json::to_value(&payload).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![cmd_bootstrap, cmd_solve])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
