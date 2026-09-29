mod commands;
mod demo;
mod dto;
mod state;
mod tray;

use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::read_state,
            commands::set_key,
            commands::apply_keymap,
            commands::preview_backlight,
            commands::apply_backlight,
            commands::list_profiles,
            commands::save_profile,
            commands::apply_profile,
            commands::duplicate_profile,
            commands::delete_profile,
            commands::export_profile,
            commands::import_profile,
            commands::battery,
            commands::catalog,
            commands::legends,
            commands::factory_backup_info,
        ])
        .setup(|app| {
            tray::setup(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running openlofree");
}
