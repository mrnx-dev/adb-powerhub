use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::time::Duration;
use tauri::State;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::AppState;

const ADB_TIMEOUT_SECS: u64 = 15;

fn run_adb_cmd(adb_path: &str, args: &[&str]) -> Result<String, String> {
    run_adb_cmd_with_timeout(adb_path, args, ADB_TIMEOUT_SECS)
}

fn run_adb_cmd_with_timeout(adb_path: &str, args: &[&str], timeout_secs: u64) -> Result<String, String> {
    use std::sync::mpsc;
    use std::thread;

    let adb = adb_path.to_string();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let timeout_secs_clone = timeout_secs;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        #[cfg(windows)]
        let result = Command::new(&adb)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(0x08000000)
            .output();

        #[cfg(not(windows))]
        let result = Command::new(&adb)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(timeout_secs_clone)) {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if output.status.success() {
                Ok(stdout)
            } else if stderr.is_empty() {
                Err(format!("Command failed with status {}", output.status))
            } else {
                Err(stderr)
            }
        }
        Ok(Err(e)) => Err(format!("Failed to execute adb: {}", e)),
        Err(_) => Err(format!("ADB command timed out after {} seconds", timeout_secs_clone)),
    }
}

fn run_adb_cmd_with_device(adb_path: &str, device_serial: Option<&str>, args: &[&str]) -> Result<String, String> {
    run_adb_cmd_with_device_timed(adb_path, device_serial, args, ADB_TIMEOUT_SECS)
}

fn run_adb_cmd_with_device_timed(adb_path: &str, device_serial: Option<&str>, args: &[&str], timeout_secs: u64) -> Result<String, String> {
    let mut full_args: Vec<&str> = Vec::new();
    if let Some(serial) = device_serial {
        full_args.push("-s");
        full_args.push(serial);
    }
    full_args.extend_from_slice(args);
    run_adb_cmd_with_timeout(adb_path, &full_args, timeout_secs)
}

fn get_adb_path(state: &State<AppState>) -> String {
    state.adb_path.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn get_device_serial(state: &State<AppState>) -> Option<String> {
    state.connected_device.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

struct CommandGuard<'a> {
    _guard: tokio::sync::MutexGuard<'a, ()>,
}

impl<'a> CommandGuard<'a> {
    async fn acquire(state: &'a State<'_, AppState>) -> Result<Self, String> {
        let guard = state.command_lock.lock().await;
        Ok(CommandGuard { _guard: guard })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub state: String,
    pub model: String,
    pub product: String,
    pub device: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BatteryInfo {
    pub level: i32,
    pub status: String,
    pub health: String,
    pub temperature: f32,
    pub plugged: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceStats {
    pub battery: BatteryInfo,
    pub cpu_usage: f32,
    pub model: String,
    pub android_version: String,
    pub sdk_version: String,
}

#[tauri::command]
pub fn adb_connect(state: State<AppState>, ip: String) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let result = run_adb_cmd(&adb, &["connect", &ip])?;
    if result.contains("connected") || result.contains("already connected") {
        let mut device = lock_state!(state.connected_device);
        *device = Some(ip.clone());
        Ok(result)
    } else {
        Err(result)
    }
}

#[tauri::command]
pub fn adb_disconnect(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let result = run_adb_cmd(&adb, &["disconnect"])?;
    let mut device = lock_state!(state.connected_device);
    *device = None;
    Ok(result)
}

#[tauri::command]
pub fn adb_devices(state: State<AppState>) -> Result<Vec<DeviceInfo>, String> {
    let adb = get_adb_path(&state);
    let output = run_adb_cmd(&adb, &["devices", "-l"])?;
    let mut devices = Vec::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let id = parts[0].to_string();
            let device_state = parts[1].to_string();
            let model = parts.iter()
                .find(|p| p.starts_with("model:"))
                .map(|p| p.strip_prefix("model:").unwrap_or("").to_string())
                .unwrap_or_default();
            let product = parts.iter()
                .find(|p| p.starts_with("product:"))
                .map(|p| p.strip_prefix("product:").unwrap_or("").to_string())
                .unwrap_or_default();
            let device = parts.iter()
                .find(|p| p.starts_with("device:"))
                .map(|p| p.strip_prefix("device:").unwrap_or("").to_string())
                .unwrap_or_default();

            devices.push(DeviceInfo {
                id,
                state: device_state,
                model,
                product,
                device,
            });
        }
    }

    Ok(devices)
}

#[tauri::command]
pub fn adb_shell(state: State<AppState>, command: String) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", &command])
}

#[tauri::command]
pub fn adb_get_battery(state: State<AppState>) -> Result<BatteryInfo, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    battery_internal(&adb, serial.as_deref())
}

fn battery_internal(adb: &str, serial: Option<&str>) -> Result<BatteryInfo, String> {
    let output = run_adb_cmd_with_device(adb, serial, &["shell", "dumpsys", "battery"])?;

    let mut level = 0;
    let mut status = String::from("Unknown");
    let mut health = String::from("Unknown");
    let mut temperature = 0.0f32;
    let mut plugged = false;

    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("level:") {
            level = line.split(':').nth(1).unwrap_or("0").trim().parse().unwrap_or(0);
        } else if line.starts_with("status:") {
            let raw = line.split(':').nth(1).unwrap_or("").trim();
            status = match raw {
                "1" => "Unknown".to_string(),
                "2" => "Charging".to_string(),
                "3" => "Discharging".to_string(),
                "4" => "Not Charging".to_string(),
                "5" => "Full".to_string(),
                other => other.to_string(),
            };
        } else if line.starts_with("health:") {
            health = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("temperature:") {
            let temp_str = line.split(':').nth(1).unwrap_or("0").trim();
            let temp_int: i32 = temp_str.parse().unwrap_or(0);
            temperature = temp_int as f32 / 10.0;
        } else if line.starts_with("AC powered:") || line.starts_with("USB powered:") || line.starts_with("Wireless powered:") {
            if line.contains("true") {
                plugged = true;
            }
        }
    }

    Ok(BatteryInfo {
        level,
        status,
        health,
        temperature,
        plugged,
    })
}

#[tauri::command]
pub async fn adb_get_cpu(state: State<'_, AppState>) -> Result<f32, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    let output1 = run_adb_cmd_with_device_timed(&adb, serial.as_deref(), &["shell", "cat", "/proc/stat"], 5)?;
    let first_line1 = output1.lines().next().unwrap_or("cpu 0 0 0 0 0 0 0");
    let vals1: Vec<u64> = first_line1
        .split_whitespace()
        .skip(1)
        .filter_map(|v| v.parse().ok())
        .collect();

    tokio::time::sleep(Duration::from_millis(500)).await;

    let output2 = run_adb_cmd_with_device_timed(&adb, serial.as_deref(), &["shell", "cat", "/proc/stat"], 5)?;
    let first_line2 = output2.lines().next().unwrap_or("cpu 0 0 0 0 0 0 0");
    let vals2: Vec<u64> = first_line2
        .split_whitespace()
        .skip(1)
        .filter_map(|v| v.parse().ok())
        .collect();

    if vals1.len() < 4 || vals2.len() < 4 {
        return Ok(0.0);
    }

    let idle1 = vals1.get(3).unwrap_or(&0);
    let idle2 = vals2.get(3).unwrap_or(&0);
    let total1: u64 = vals1.iter().sum();
    let total2: u64 = vals2.iter().sum();

    let total_diff = (total2 - total1) as f32;
    let idle_diff = (idle2 - idle1) as f32;

    if total_diff == 0.0 {
        return Ok(0.0);
    }

    let usage = ((total_diff - idle_diff) / total_diff) * 100.0;
    Ok((usage * 10.0).round() / 10.0)
}

#[tauri::command]
pub fn adb_get_device_info(state: State<AppState>) -> Result<DeviceStats, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let battery = battery_internal(&adb, serial.as_deref())?;

    let model = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "getprop", "ro.product.model"])
        .unwrap_or_default()
        .trim()
        .to_string();

    let android_version = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "getprop", "ro.build.version.release"])
        .unwrap_or_default()
        .trim()
        .to_string();

    let sdk_version = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "getprop", "ro.build.version.sdk"])
        .unwrap_or_default()
        .trim()
        .to_string();

    Ok(DeviceStats {
        battery,
        cpu_usage: 0.0,
        model,
        android_version,
        sdk_version,
    })
}

#[tauri::command]
pub fn adb_set_wifi(state: State<AppState>, enable: bool) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let cmd = if enable { "enable" } else { "disable" };
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "svc", "wifi", cmd])
}

#[tauri::command]
pub fn adb_set_data(state: State<AppState>, enable: bool) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let cmd = if enable { "enable" } else { "disable" };
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "svc", "data", cmd])
}

#[tauri::command]
pub fn adb_set_airplane(state: State<AppState>, enable: bool) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let mode = if enable { "1" } else { "0" };
    let _ = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "settings", "put", "global", "airplane_mode_on", mode]);
    let broadcast = "android.intent.action.AIRPLANE_MODE";

    if enable {
        run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "am", "broadcast", "-a", broadcast, "--ez", "state", "true"])
    } else {
        run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "am", "broadcast", "-a", broadcast, "--ez", "state", "false"])
    }
}

#[tauri::command]
pub fn adb_set_bluetooth(state: State<AppState>, enable: bool) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let cmd = if enable { "enable" } else { "disable" };
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "svc", "bluetooth", cmd])
}

#[tauri::command]
pub fn adb_show_taps(state: State<AppState>, enable: bool) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let val = if enable { "1" } else { "0" };
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "settings", "put", "system", "show_touches", val])
}

#[tauri::command]
pub fn adb_layout_bounds(state: State<AppState>, enable: bool) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let val = if enable { "1" } else { "0" };
    let _ = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "setprop", "debug.layout", val]);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "service", "call", "window", "3"])
}

#[tauri::command]
pub fn adb_stay_awake(state: State<AppState>, enable: bool) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let val = if enable { "7" } else { "0" };
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "settings", "put", "global", "stay_on_while_plugged_in", val])
}

#[tauri::command]
pub fn adb_key_home(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "3"])
}

#[tauri::command]
pub fn adb_key_back(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "4"])
}

#[tauri::command]
pub fn adb_key_recent(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "187"])
}

#[tauri::command]
pub fn adb_key_vol_up(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "24"])
}

#[tauri::command]
pub fn adb_key_vol_down(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "25"])
}

#[tauri::command]
pub fn adb_key_mute(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "164"])
}

#[tauri::command]
pub fn adb_key_power(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "26"])
}

#[tauri::command]
pub fn adb_key_prev(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "88"])
}

#[tauri::command]
pub fn adb_key_play_pause(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "85"])
}

#[tauri::command]
pub fn adb_key_next(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "87"])
}

fn encode_adb_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 3);
    for c in text.chars() {
        match c {
            ' ' => result.push_str("%s"),
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' => result.push(c),
            _ => {
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                for byte in s.bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result
}

#[tauri::command]
pub fn adb_input_text(state: State<AppState>, text: String) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let encoded = encode_adb_text(&text);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "text", &encoded])
}

#[tauri::command]
pub fn adb_set_brightness(state: State<AppState>, value: i32) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let clamped = value.clamp(0, 255);
    let val_str = clamped.to_string();
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "settings", "put", "system", "screen_brightness", &val_str])
}

#[tauri::command]
pub fn adb_reboot_recovery(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["reboot", "recovery"])
}

#[tauri::command]
pub fn adb_reboot_bootloader(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["reboot", "bootloader"])
}

#[tauri::command]
pub fn adb_screenshot(state: State<AppState>, save_dir: Option<String>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let timestamp = chrono_or_simple_timestamp();
    let device_path = format!("/sdcard/adb_powerhub_screenshot_{}.png", timestamp);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "screencap", "-p", &device_path])?;

    let local_dir = save_dir.unwrap_or_else(|| dirs_or_fallback_screenshots());
    let local_file = std::path::PathBuf::from(&local_dir)
        .join(format!("screenshot_{}.png", timestamp))
        .to_string_lossy()
        .to_string();

    if std::fs::create_dir_all(&local_dir).is_err() {
        let _ = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "rm", &device_path]);
        return Err(format!("Cannot create directory: {}", local_dir));
    }

    let pull_result = run_adb_cmd_with_device(&adb, serial.as_deref(), &["pull", &device_path, &local_file]);
    let _ = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "rm", &device_path]);

    if pull_result.is_err() {
        return Err(format!("Screenshot pull failed: {}", pull_result.unwrap_err()));
    }
    Ok(local_file)
}

#[tauri::command]
pub fn adb_reboot(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["reboot"])
}

#[tauri::command]
pub fn adb_custom_cmd(state: State<AppState>, command: String) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let args: Vec<&str> = command.split_whitespace().collect();
    if args.is_empty() {
        return Err("Empty command".to_string());
    }
    let cmd_args = if args[0] == "adb" { &args[1..] } else { &args[..] };
    if let Some(ref s) = serial {
        let mut full_args: Vec<&str> = vec!["-s", s];
        full_args.extend_from_slice(cmd_args);
        run_adb_cmd(&adb, &full_args)
    } else {
        run_adb_cmd(&adb, cmd_args)
    }
}

#[tauri::command]
pub fn adb_auto_connect(state: State<AppState>) -> Result<Vec<DeviceInfo>, String> {
    let devices = adb_devices(state.clone())?;
    Ok(devices)
}

#[tauri::command]
pub fn adb_set_device(state: State<AppState>, device_id: String) -> Result<(), String> {
    let mut device = lock_state!(state.connected_device);
    *device = Some(device_id);
    Ok(())
}

#[tauri::command]
pub async fn adb_poll_device_stats(state: State<'_, AppState>) -> Result<DeviceStats, String> {
    let _guard = CommandGuard::acquire(&state).await?;
    let battery = adb_get_battery(state.clone())?;
    let cpu = adb_get_cpu(state.clone()).await?;
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    let model = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "getprop", "ro.product.model"])
        .unwrap_or_default()
        .trim()
        .to_string();

    let android_version = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "getprop", "ro.build.version.release"])
        .unwrap_or_default()
        .trim()
        .to_string();

    let sdk_version = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "getprop", "ro.build.version.sdk"])
        .unwrap_or_default()
        .trim()
        .to_string();

    Ok(DeviceStats {
        battery,
        cpu_usage: cpu,
        model,
        android_version,
        sdk_version,
    })
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ToggleStates {
    pub wifi: bool,
    pub data: bool,
    pub airplane: bool,
    pub bluetooth: bool,
    pub show_taps: bool,
    pub layout_bounds: bool,
    pub stay_awake: bool,
    pub brightness: i32,
}

fn parse_bool(output: &str) -> bool {
    let val = output.trim();
    val == "1" || val == "true" || val == "7"
}

#[tauri::command]
pub fn adb_sync_toggles(state: State<AppState>) -> Result<ToggleStates, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let s = serial.as_deref();

    let wifi_output = run_adb_cmd_with_device(&adb, s, &["shell", "settings", "get", "global", "wifi_on"]).unwrap_or_default();
    let data_output = run_adb_cmd_with_device(&adb, s, &["shell", "settings", "get", "global", "mobile_data"]).unwrap_or_default();
    let airplane_output = run_adb_cmd_with_device(&adb, s, &["shell", "settings", "get", "global", "airplane_mode_on"]).unwrap_or_default();
    let bt_output = run_adb_cmd_with_device(&adb, s, &["shell", "settings", "get", "global", "bluetooth_on"]).unwrap_or_default();
    let taps_output = run_adb_cmd_with_device(&adb, s, &["shell", "settings", "get", "system", "show_touches"]).unwrap_or_default();
    let layout_output = run_adb_cmd_with_device(&adb, s, &["shell", "getprop", "debug.layout"]).unwrap_or_default();
    let awake_output = run_adb_cmd_with_device(&adb, s, &["shell", "settings", "get", "global", "stay_on_while_plugged_in"]).unwrap_or_default();
    let brightness_output = run_adb_cmd_with_device(&adb, s, &["shell", "settings", "get", "system", "screen_brightness"]).unwrap_or_default();
    let brightness_val: i32 = brightness_output.trim().parse().unwrap_or(128).clamp(0, 255);

    Ok(ToggleStates {
        wifi: parse_bool(&wifi_output),
        data: parse_bool(&data_output),
        airplane: parse_bool(&airplane_output),
        bluetooth: parse_bool(&bt_output),
        show_taps: parse_bool(&taps_output),
        layout_bounds: parse_bool(&layout_output),
        stay_awake: parse_bool(&awake_output),
        brightness: brightness_val,
    })
}

#[tauri::command]
pub fn adb_rotate(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let _ = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "settings", "put", "system", "accelerometer_rotation", "0"]);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "276"])
}

fn chrono_or_simple_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn dirs_or_fallback_screenshots() -> String {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home)
        .join("Pictures")
        .join("adb-powerhub")
        .to_string_lossy()
        .to_string()
}