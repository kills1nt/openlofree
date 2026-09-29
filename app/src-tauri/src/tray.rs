//! System tray: battery line, one-click profiles, show window, quit.
//! The menu is rebuilt when profiles change and once a minute for the battery.
use std::time::Duration;

use flow2_core::battery;
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::state::AppState;

const TRAY_ID: &str = "main";
const MAX_PROFILES_IN_MENU: usize = 12;

pub fn battery_label(level: Option<u8>) -> String {
    match level {
        Some(p) => format!("Battery {p}%"),
        None => "Battery unknown".to_string(),
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let state = app.state::<AppState>();
    let disabled =
        |id: &str, text: String| MenuItemBuilder::with_id(id, text).enabled(false).build(app);

    let mut menu = MenuBuilder::new(app)
        .item(&disabled("title", "openlofree".into())?)
        .item(&disabled("battery", battery_label(state.battery()))?)
        .separator();

    let profiles = state.store.list().unwrap_or_default();
    if profiles.is_empty() {
        menu = menu.item(&disabled("no-profiles", "No saved profiles".into())?);
    }
    for p in profiles.iter().take(MAX_PROFILES_IN_MENU) {
        let item =
            MenuItemBuilder::with_id(format!("profile:{}", p.name), format!("Apply {}", p.name))
                .build(app)?;
        menu = menu.item(&item);
    }

    menu.separator()
        .item(&MenuItemBuilder::with_id("show", "Show window").build(app)?)
        .item(&MenuItemBuilder::with_id("quit", "Quit").build(app)?)
        .build()
}

/// Rebuilds the tray menu. Safe to call from any thread.
pub fn refresh(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let (Some(tray), Ok(menu)) = (handle.tray_by_id(TRAY_ID), build_menu(&handle)) {
            let _ = tray.set_menu(Some(menu));
        }
    });
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn handle_menu(app: &AppHandle, id: &str) {
    match id {
        "show" => show_main(app),
        "quit" => app.exit(0),
        _ => {
            if let Some(name) = id.strip_prefix("profile:") {
                let (app, name) = (app.clone(), name.to_string());
                std::thread::spawn(move || {
                    let result = app.state::<AppState>().apply_profile(&name);
                    match result {
                        Ok(changed) => {
                            let _ = app.emit(
                                "profile-applied",
                                serde_json::json!({ "name": name, "changed": changed }),
                            );
                        }
                        Err(e) => {
                            let _ = app.emit("tray-error", e.message);
                        }
                    }
                });
            }
        }
    }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("openlofree")
        .menu(&menu)
        .show_menu_on_left_click(false);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder
        .on_menu_event(|app, event| handle_menu(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;

    let handle = app.clone();
    std::thread::spawn(move || loop {
        handle.state::<AppState>().set_battery(battery::read());
        refresh(&handle);
        std::thread::sleep(Duration::from_secs(60));
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_label_reads_plainly() {
        assert_eq!(battery_label(Some(87)), "Battery 87%");
        assert_eq!(battery_label(None), "Battery unknown");
    }
}
