#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_export]
macro_rules! lock_state {
    ($mutex:expr) => {
        $mutex.lock().map_err(|e| format!("Lock poisoned: {}", e))?
    };
}

mod adb;
mod scrcpy;
mod settings;

use adb::*;
use scrcpy::*;
use settings::*;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct AppState {
    pub adb_path: Mutex<String>,
    pub connected_device: Mutex<Option<String>>,
    pub scrcpy_path: Mutex<Option<String>>,
    pub scrcpy_process: Mutex<Option<u32>>,
    pub cancel_download: Mutex<Arc<AtomicBool>>,
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    opener::open(&path).map_err(|e| format!("Failed to open folder: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            adb_path: Mutex::new("adb".to_string()),
            connected_device: Mutex::new(None),
            scrcpy_path: Mutex::new(None),
            scrcpy_process: Mutex::new(None),
            cancel_download: Mutex::new(Arc::new(AtomicBool::new(false))),
        })
        .invoke_handler(tauri::generate_handler![
            open_folder,
            adb_connect,
            adb_disconnect,
            adb_devices,
            adb_shell,
            adb_get_battery,
            adb_get_cpu,
            adb_get_device_info,
            adb_set_wifi,
            adb_set_data,
            adb_set_airplane,
            adb_set_bluetooth,
            adb_show_taps,
            adb_layout_bounds,
            adb_stay_awake,
            adb_key_home,
            adb_key_back,
            adb_key_recent,
            adb_key_vol_up,
            adb_key_vol_down,
            adb_key_mute,
            adb_key_power,
            adb_key_prev,
            adb_key_play_pause,
            adb_key_next,
            adb_input_text,
            adb_set_brightness,
            adb_reboot_recovery,
            adb_reboot_bootloader,
            adb_screenshot,
            adb_reboot,
            adb_custom_cmd,
            adb_rotate,
            adb_sync_toggles,
            adb_find_scrcpy,
            adb_launch_scrcpy,
            adb_stop_scrcpy,
            adb_auto_connect,
            adb_set_device,
            adb_poll_device_stats,
            settings_set_adb_path,
            settings_set_scrcpy_path,
            settings_validate_adb,
            settings_validate_scrcpy,
            settings_detect_adb,
            settings_get_download_info,
            settings_download_adb,
            settings_cancel_download,
            settings_get_app_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}