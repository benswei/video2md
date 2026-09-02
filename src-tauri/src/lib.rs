mod commands;
mod project;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::set_current_project,
            commands::get_current_project,
            commands::get_platform_capabilities,
            commands::import_files,
            commands::import_online_sources,
            commands::remove_source,
            commands::clear_sources,
            commands::list_sources,
            commands::list_artifacts,
            commands::get_config,
            commands::set_config,
            commands::run_transcribe,
            commands::cancel_task,
            commands::retry_task,
            commands::read_text_file,
            commands::open_file,
            commands::show_in_folder,
            commands::get_app_dir,
            commands::check_dependencies,
            commands::pick_import_files,
            commands::pick_document_files,
            commands::pick_cookie_file,
            commands::start_bilibili_qr_login,
            commands::poll_bilibili_qr_login,
            commands::pick_import_folder,
            commands::run_document_reorganize,
            commands::list_document_tasks,
            commands::list_tasks,
            commands::clear_tasks,
            commands::pick_export_dir,
            commands::export_artifacts,
            commands::read_file_base64,
            commands::test_connection,
            commands::test_ai_connection,
            commands::rename_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
