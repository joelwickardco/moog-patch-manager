mod commands;
mod db;
mod models;
mod moog;
mod utils;

use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<db::Database>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize database in app data directory
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("patches.db");
            let database = db::Database::new(&db_path).expect("Failed to initialize database");

            app.manage(AppState {
                db: Mutex::new(database),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Library commands
            commands::libraries::get_all_libraries,
            commands::libraries::get_library_by_id,
            commands::libraries::create_library,
            commands::libraries::update_library,
            commands::libraries::delete_library,
            // Patch commands
            commands::patches::get_all_patches,
            commands::patches::get_patches_for_library,
            commands::patches::get_patch_by_id,
            commands::patches::toggle_favorite,
            commands::patches::update_patch_notes,
            commands::patches::delete_patch,
            commands::patches::search_patches,
            // Sequence commands
            commands::sequences::get_all_sequences,
            commands::sequences::get_sequence_by_id,
            commands::sequences::update_sequence_notes,
            commands::sequences::delete_sequence,
            commands::sequences::search_sequences,
            // Category commands
            commands::categories::get_all_categories,
            commands::categories::create_category,
            commands::categories::update_category,
            commands::categories::delete_category,
            commands::categories::assign_patch_to_category,
            commands::categories::remove_patch_from_category,
            commands::categories::assign_sequence_to_category,
            commands::categories::remove_sequence_from_category,
            // Bank commands
            commands::banks::get_banks_for_library,
            commands::banks::get_bank_by_number,
            commands::banks::update_bank_name,
            commands::banks::assign_patch_to_slot,
            commands::banks::assign_sequence_to_slot,
            commands::banks::clear_patch_slot,
            commands::banks::clear_sequence_slot,
            // Import/Export commands
            commands::import::import_library_zip,
            commands::import::import_bank_directory,
            commands::import::validate_library_structure,
            commands::export::export_library,
            commands::export::preview_export,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
