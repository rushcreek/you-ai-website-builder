

mod commands;
mod credentials;
mod github;
mod cloudflare;
mod llm;
mod scraper;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_setup_status,
            commands::save_credential,
            commands::remove_credential,
            commands::generate_oauth_url,
            commands::exchange_oauth_code,
            commands::test_connection,
            commands::get_llm_providers,
            commands::chat_with_llm,
            commands::scrape_site,
            commands::create_project,
            commands::list_projects,
            commands::open_project,
            commands::publish_site,
            commands::get_site_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
