use end_web::{BootstrapPayload, Lang, SolvePayload, bootstrap, solve_from_aic_toml};

fn parse_lang(tag: &str) -> Result<Lang, String> {
  match tag.trim().to_ascii_lowercase().as_str() {
    "zh" => Ok(Lang::Zh),
    "en" => Ok(Lang::En),
    other => Err(format!("unknown lang `{other}` (expected `zh` or `en`)")),
  }
}

#[tauri::command]
fn cmd_bootstrap(lang: String) -> Result<BootstrapPayload, String> {
  let lang = parse_lang(&lang)?;
  bootstrap(lang).map_err(|error| error.to_string())
}

#[tauri::command]
fn cmd_solve(lang: String, aic_toml: String) -> Result<SolvePayload, String> {
  let lang = parse_lang(&lang)?;
  solve_from_aic_toml(lang, &aic_toml).map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![cmd_bootstrap, cmd_solve])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
