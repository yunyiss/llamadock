mod commands;
mod config;
mod fsutil;
mod parser;
mod process;
mod watch;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::config::get_config,
            commands::workspace::pick_work_dir,
            commands::workspace::scan_scripts,
            commands::workspace::open_folder,
            commands::workspace::path_exists,
            commands::script::parse_script,
            commands::script::parse_text,
            commands::script::rebuild_script,
            commands::script::save_script,
            commands::engine::list_engines,
            commands::engine::save_engine,
            commands::engine::remove_engine,
            commands::engine::set_current_engine,
            commands::engine::pick_engine_file,
            commands::engine::switch_engine_text,
            commands::engine::switch_engine_all,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
