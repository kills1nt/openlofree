//! Tauri commands. Thin: each one takes the session lock, calls `flow2-core`, and returns plain data.
use std::path::Path;

use flow2_core::backlight::{self, Backlight, Mode};
use flow2_core::battery;
use flow2_core::keycodes::{self, CatalogGroup};
use flow2_core::keymap::{self, Keymap};
use flow2_core::profile::{self, Profile, ProfileInfo};
use tauri::{AppHandle, State};

use crate::demo::unique_name;
use crate::dto::{AppError, BackupInfo, ConnectInfo, StateDto};
use crate::state::AppState;
use crate::tray;

type R<T> = Result<T, AppError>;

#[tauri::command]
pub async fn connect(app: AppHandle, state: State<'_, AppState>, demo: bool) -> R<ConnectInfo> {
    let info = state.connect(demo)?;
    tray::refresh(&app);
    Ok(info)
}

#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> R<()> {
    state.disconnect();
    Ok(())
}

#[tauri::command]
pub async fn read_state(state: State<'_, AppState>) -> R<StateDto> {
    state.with(|s| {
        let km = keymap::read(&mut s.client, s.def.rows, s.def.cols)?;
        let backlight = backlight::read(&mut s.client)?;
        Ok(StateDto {
            layers: km.layers,
            backlight,
        })
    })
}

#[tauri::command]
pub async fn set_key(state: State<'_, AppState>, layer: u8, row: u8, col: u8, code: u16) -> R<()> {
    state.with(|s| {
        keymap::check_position(s.def.rows, s.def.cols, s.layers, layer, row, col)?;
        s.guard_write()?;
        keymap::set_key_verified(&mut s.client, layer, row, col, code)
    })
}

/// Writes only the keys that differ from the keyboard. Returns how many were written.
#[tauri::command]
pub async fn apply_keymap(state: State<'_, AppState>, layers: Vec<Vec<u16>>) -> R<usize> {
    state.with(|s| {
        s.guard_write()?;
        let target = Keymap {
            rows: s.def.rows,
            cols: s.def.cols,
            layers,
        };
        keymap::apply(&mut s.client, &target)
    })
}

/// Live preview: changes the light now, does not save. The guard runs first so the factory backup
/// never captures a previewed light.
#[tauri::command]
pub async fn preview_backlight(state: State<'_, AppState>, mode: Mode, brightness: u8) -> R<()> {
    state.with(|s| {
        s.guard_write()?;
        backlight::preview(&mut s.client, Backlight { mode, brightness })
    })
}

#[tauri::command]
pub async fn apply_backlight(state: State<'_, AppState>, mode: Mode, brightness: u8) -> R<()> {
    state.with(|s| {
        s.guard_write()?;
        backlight::apply(&mut s.client, Backlight { mode, brightness })
    })
}

#[tauri::command]
pub async fn list_profiles(state: State<'_, AppState>) -> R<Vec<ProfileInfo>> {
    Ok(state.store.list()?)
}

#[tauri::command]
pub async fn save_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    layers: Vec<Vec<u16>>,
    backlight: Backlight,
) -> R<Vec<ProfileInfo>> {
    let p = state.build_profile(&name, layers, backlight)?;
    state.store.save(&p)?;
    tray::refresh(&app);
    Ok(state.store.list()?)
}

#[tauri::command]
pub async fn apply_profile(state: State<'_, AppState>, name: String) -> R<usize> {
    state.apply_profile(&name)
}

#[tauri::command]
pub async fn duplicate_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    new_name: String,
) -> R<Vec<ProfileInfo>> {
    state.store.duplicate(&name, &new_name)?;
    tray::refresh(&app);
    Ok(state.store.list()?)
}

#[tauri::command]
pub async fn delete_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> R<Vec<ProfileInfo>> {
    state.store.delete(&name)?;
    tray::refresh(&app);
    Ok(state.store.list()?)
}

/// `path` comes from the native save dialog.
#[tauri::command]
pub async fn export_profile(state: State<'_, AppState>, name: String, path: String) -> R<()> {
    Ok(state.store.load(&name)?.save(Path::new(&path))?)
}

/// `path` comes from the native open dialog. A name clash never overwrites a stored profile.
#[tauri::command]
pub async fn import_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> R<Vec<ProfileInfo>> {
    let mut p = Profile::load(Path::new(&path))?;
    let names: Vec<String> = state.store.list()?.into_iter().map(|i| i.name).collect();
    p.name = unique_name(&names, &p.name);
    state.store.save(&p)?;
    tray::refresh(&app);
    Ok(state.store.list()?)
}

#[tauri::command]
pub async fn battery(state: State<'_, AppState>) -> R<Option<u8>> {
    let level = battery::read();
    state.set_battery(level);
    Ok(level)
}

#[tauri::command]
pub async fn catalog(state: State<'_, AppState>) -> R<Vec<CatalogGroup>> {
    state.with(|s| Ok(keycodes::catalog(s.variant, s.layers)))
}

#[tauri::command]
pub async fn legends(state: State<'_, AppState>, codes: Vec<u16>) -> R<Vec<String>> {
    state.with(|s| {
        Ok(codes
            .iter()
            .map(|&c| keycodes::legend(c, s.variant))
            .collect())
    })
}

#[tauri::command]
pub async fn factory_backup_info(state: State<'_, AppState>) -> R<BackupInfo> {
    state.with(|s| {
        let path = profile::factory_backup_path(&s.model_id);
        Ok(BackupInfo {
            exists: path.as_ref().is_some_and(|p| p.exists()),
            path: path.map(|p| p.display().to_string()),
        })
    })
}
