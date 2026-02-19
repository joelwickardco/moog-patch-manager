mod commands;
mod db;
mod models;
mod moog;
mod utils;

use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Emitter, Manager};

pub struct AppState {
    pub db: Mutex<db::Database>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Build native application menu
            let check_updates = MenuItem::with_id(
                app,
                "check_updates",
                "Check for Updates...",
                true,
                None::<&str>,
            )?;

            #[cfg(target_os = "macos")]
            {
                let app_menu = Submenu::with_items(
                    app,
                    "Muse Patch Manager",
                    true,
                    &[
                        &PredefinedMenuItem::about(app, None, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &check_updates,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::hide(app, None)?,
                        &PredefinedMenuItem::hide_others(app, None)?,
                        &PredefinedMenuItem::show_all(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::quit(app, None)?,
                    ],
                )?;

                let edit_menu = Submenu::with_items(
                    app,
                    "Edit",
                    true,
                    &[
                        &PredefinedMenuItem::cut(app, None)?,
                        &PredefinedMenuItem::copy(app, None)?,
                        &PredefinedMenuItem::paste(app, None)?,
                        &PredefinedMenuItem::select_all(app, None)?,
                    ],
                )?;

                let window_menu = Submenu::with_items(
                    app,
                    "Window",
                    true,
                    &[
                        &PredefinedMenuItem::minimize(app, None)?,
                        &PredefinedMenuItem::close_window(app, None)?,
                    ],
                )?;

                let menu = Menu::with_items(app, &[&app_menu, &edit_menu, &window_menu])?;
                app.set_menu(menu)?;
            }

            #[cfg(not(target_os = "macos"))]
            {
                let help_menu = Submenu::with_items(app, "Help", true, &[&check_updates])?;

                let menu = Menu::with_items(app, &[&help_menu])?;
                app.set_menu(menu)?;
            }

            app.on_menu_event(|app, event| {
                if event.id() == "check_updates" {
                    let _ = app.emit("menu:check-for-updates", ());
                }
            });

            // Initialize database in app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
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
            // Tag commands
            commands::patches::get_all_tags,
            commands::patches::add_tag_to_patch,
            commands::patches::remove_tag_from_patch,
            commands::patches::update_patch_tags,
            commands::patches::get_tag_usage_counts,
            commands::patches::delete_unused_tags,
            // Sequence commands
            commands::sequences::get_all_sequences,
            commands::sequences::get_sequence_by_id,
            commands::sequences::update_sequence_notes,
            commands::sequences::delete_sequence,
            commands::sequences::search_sequences,
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
