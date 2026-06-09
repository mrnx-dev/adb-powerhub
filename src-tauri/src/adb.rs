use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager, State};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::AppState;

const ADB_TIMEOUT_SECS: u64 = 15;

/// Embedded DEX bytecode for on-device label resolution.
/// Generated from `src-tauri/assets/AppLabels.java` using D8.
const APP_LABELS_DEX: &[u8] = include_bytes!("../assets/app_labels.dex");
const DEX_REMOTE_PATH: &str = "/data/local/tmp/adbph_labels.dex";

/// Embedded DEX for on-device icon extraction via ZIP reading.
/// Generated from AppIcons.java using D8 (r8.jar).
const APP_ICONS_DEX: &[u8] = include_bytes!("../assets/app_icons.dex");
const ICONS_DEX_REMOTE: &str = "/data/local/tmp/adbph_icons.dex";
/// Timeout for on-device DEX execution.
const ICONS_DEX_TIMEOUT_SECS: u64 = 120;

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

fn kill_process_tree(pid: u32) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .creation_flags(0x08000000)
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .output();
    }
    #[cfg(unix)]
    {
        unsafe {
            libc::kill(-(pid as i32), 9);
        }
    }
}

fn run_adb_cmd_cancellable(
    cancel_flag: &AtomicBool,
    process_slot: &Arc<std::sync::Mutex<Option<u32>>>,
    adb_path: &str,
    args: &[&str],
    timeout_secs: u64,
) -> Result<String, String> {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    let adb = adb_path.to_string();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    #[cfg(windows)]
    let child = Command::new(&adb)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("Failed to spawn adb: {}", e))?;

    #[cfg(not(windows))]
    let child = Command::new(&adb)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn adb: {}", e))?;

    let pid = child.id();
    {
        let mut slot = process_slot.lock().unwrap();
        *slot = Some(pid);
    }

    let tx_child = tx.clone();
    std::thread::spawn(move || {
        let output = child.wait_with_output();
        let _ = tx_child.send(output);
    });

    let start = std::time::Instant::now();
    loop {
        if cancel_flag.load(Ordering::SeqCst) {
            kill_process_tree(pid);
            {
                let mut slot = process_slot.lock().unwrap();
                *slot = None;
            }
            return Err("cancelled".to_string());
        }

        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Ok(output)) => {
                {
                    let mut slot = process_slot.lock().unwrap();
                    *slot = None;
                }
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return if output.status.success() {
                    Ok(stdout)
                } else if stderr.is_empty() {
                    Err(format!("Command failed with status {}", output.status))
                } else {
                    Err(stderr)
                };
            }
            Ok(Err(e)) => {
                {
                    let mut slot = process_slot.lock().unwrap();
                    *slot = None;
                }
                return Err(format!("Failed to execute adb: {}", e));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if start.elapsed() > Duration::from_secs(timeout_secs) {
                    kill_process_tree(pid);
                    {
                        let mut slot = process_slot.lock().unwrap();
                        *slot = None;
                    }
                    return Err(format!("ADB command timed out after {} seconds", timeout_secs));
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                {
                    let mut slot = process_slot.lock().unwrap();
                    *slot = None;
                }
                return Err("ADB monitor thread disconnected".to_string());
            }
        }
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
    pub transport: String,
}

fn detect_transport(id: &str) -> String {
    if id.contains(':') { "wifi".to_string() } else { "usb".to_string() }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BatteryInfo {
    pub level: i32,
    pub status: String,
    pub health: String,
    pub temperature: f32,
    pub plugged: bool,
    pub voltage: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceStats {
    pub battery: BatteryInfo,
    pub cpu_usage: f32,
    pub model: String,
    pub android_version: String,
    pub sdk_version: String,
    pub ram_total_mb: u64,
    pub ram_available_mb: u64,
    pub storage_total_gb: f64,
    pub storage_used_gb: f64,
    pub screen_width: u32,
    pub screen_height: u32,
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

            let transport = detect_transport(&id);

            devices.push(DeviceInfo {
                id,
                state: device_state,
                model,
                product,
                device,
                transport,
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
    let mut voltage = 0i32;

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
        } else if line.starts_with("voltage:") {
            voltage = line.split(':').nth(1).unwrap_or("0").trim().parse().unwrap_or(0);
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
        voltage,
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
        ram_total_mb: 0,
        ram_available_mb: 0,
        storage_total_gb: 0.0,
        storage_used_gb: 0.0,
        screen_width: 0,
        screen_height: 0,
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
pub fn adb_tcpip(state: State<AppState>, serial: String, port: u16) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let port_str = port.to_string();
    run_adb_cmd(&adb, &["-s", &serial, "tcpip", &port_str])
}

#[tauri::command]
pub fn adb_get_ip(state: State<AppState>, serial: String) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let adb_ref = &adb;
    let s = Some(serial.as_str());

    let methods: &[&[&str]] = &[
        &["shell", "ip", "route"],
        &["shell", "getprop", "dhcp.wlan0.ipaddress"],
        &["shell", "ifconfig", "wlan0"],
        &["shell", "dumpsys", "connectivity"],
    ];

    for args in methods {
        let output = run_adb_cmd_with_device(adb_ref, s, args).unwrap_or_default();
        if let Some(ip) = extract_ip_from_output(&output) {
            if !ip.is_empty() {
                return Ok(ip);
            }
        }
    }
    Err("Could not determine device IP".to_string())
}

fn extract_ip_from_output(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(src) = trimmed.find("src ") {
            let ip_candidate = trimmed[src + 4..].split_whitespace().next().unwrap_or("");
            if is_valid_ip(ip_candidate) {
                return Some(ip_candidate.to_string());
            }
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        for part in &parts {
            if is_valid_ip(part) {
                return Some(part.to_string());
            }
        }
        if let Some(addr) = trimmed.find("inet addr:") {
            let ip_candidate = trimmed[addr + 10..].split_whitespace().next().unwrap_or("");
            if is_valid_ip(ip_candidate) {
                return Some(ip_candidate.to_string());
            }
        }
        if let Some(addr) = trimmed.find("LinkAddresses: [") {
            let rest = &trimmed[addr + 15..];
            if let Some(end) = rest.find(']') {
                let ip_candidate = &rest[..end];
                if is_valid_ip(ip_candidate) {
                    return Some(ip_candidate.to_string());
                }
            }
        }
    }
    None
}

fn is_valid_ip(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 { return false; }
    parts.iter().all(|p| p.parse::<u16>().map_or(false, |n| n <= 255))
}

#[tauri::command]
pub async fn adb_connect_port(state: State<'_, AppState>, ip: String, port: u16) -> Result<String, String> {
    state.cancel_connect.store(false, Ordering::SeqCst);

    let cancel = state.cancel_connect.clone();
    let process_slot = state.connect_process.clone();
    let adb = get_adb_path(&state);
    let target = format!("{}:{}", ip, port);
    let target_for_state = target.clone();

    let result = tokio::task::spawn_blocking(move || {
        run_adb_cmd_cancellable(
            &cancel,
            &process_slot,
            &adb,
            &["connect", &target],
            10,
        )
    })
    .await
    .map_err(|e| format!("Task error: {}", e))??;

    if state.cancel_connect.load(Ordering::SeqCst) {
        return Err("cancelled".to_string());
    }

    if result.contains("connected") || result.contains("already connected") {
        let mut device = lock_state!(state.connected_device);
        *device = Some(target_for_state);
        Ok(result)
    } else {
        Err(result)
    }
}

#[tauri::command]
pub async fn adb_cancel_connect(state: State<'_, AppState>) -> Result<(), String> {
    state.cancel_connect.store(true, Ordering::SeqCst);

    let pid = state.connect_process.lock().unwrap().take();
    if let Some(pid) = pid {
        kill_process_tree(pid);
    }

    *lock_state!(state.connected_device) = None;

    let adb = get_adb_path(&state);
    tokio::task::spawn_blocking(move || {
        let _ = run_adb_cmd(&adb, &["disconnect"]);
    });

    Ok(())
}

#[tauri::command]
pub fn adb_pair(state: State<AppState>, ip: String, pair_port: u16, code: String) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let target = format!("{}:{}", ip, pair_port);
    run_adb_cmd_with_timeout(&adb, &["pair", &target, &code], 15)
}

#[tauri::command]
fn parse_meminfo(output: &str) -> (u64, u64) {
    let mut total_mb = 0u64;
    let mut avail_mb = 0u64;
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("MemTotal:") {
            let kb: u64 = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            total_mb = kb / 1024;
        } else if line.starts_with("MemAvailable:") {
            let kb: u64 = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            avail_mb = kb / 1024;
        }
    }
    (total_mb, avail_mb)
}

fn parse_df_data(output: &str) -> (f64, f64) {
    // df /data output has header line + data line
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() < 2 {
        return (0.0, 0.0);
    }
    let data_line = lines[1];
    let parts: Vec<&str> = data_line.split_whitespace().collect();
    if parts.len() < 3 {
        return (0.0, 0.0);
    }
    let total_kb: f64 = parts[1].parse().unwrap_or(0.0);
    let used_kb: f64 = parts[2].parse().unwrap_or(0.0);
    let total_gb = (total_kb / 1048576.0 * 10.0).round() / 10.0;
    let used_gb = (used_kb / 1048576.0 * 10.0).round() / 10.0;
    (total_gb, used_gb)
}

fn parse_wm_size(output: &str) -> (u32, u32) {
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("Physical size:") {
            let size_str = line.split(':').nth(1).unwrap_or("").trim();
            for sep in &['x', 'X'] {
                let dims: Vec<&str> = size_str.split(*sep).collect();
                if dims.len() == 2 {
                    let w: u32 = dims[0].parse().unwrap_or(0);
                    let h: u32 = dims[1].parse().unwrap_or(0);
                    if w > 0 && h > 0 {
                        return (w, h);
                    }
                }
            }
        }
    }
    (0, 0)
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

    // RAM info
    let meminfo_output = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "cat", "/proc/meminfo"])
        .unwrap_or_default();
    let (ram_total_mb, ram_available_mb) = parse_meminfo(&meminfo_output);

    // Storage info
    let df_output = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "df", "/data"])
        .unwrap_or_default();
    let (storage_total_gb, storage_used_gb) = parse_df_data(&df_output);

    // Screen resolution
    let wm_size_output = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "wm", "size"])
        .unwrap_or_default();
    let (screen_width, screen_height) = parse_wm_size(&wm_size_output);

    Ok(DeviceStats {
        battery,
        cpu_usage: cpu,
        model,
        android_version,
        sdk_version,
        ram_total_mb,
        ram_available_mb,
        storage_total_gb,
        storage_used_gb,
        screen_width,
        screen_height,
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
    pub density: i32,
    pub density_override: Option<i32>,
    pub density_physical: i32,
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

    let density_output = run_adb_cmd_with_device(&adb, s, &["shell", "wm", "density"]).unwrap_or_default();
    let mut density_physical: i32 = 0;
    let mut density_override: Option<i32> = None;
    for line in density_output.lines() {
        let line = line.trim();
        if line.starts_with("Physical density:") {
            density_physical = line.split(':').nth(1)
                .unwrap_or("0").trim()
                .parse().unwrap_or(0);
        } else if line.starts_with("Override density:") {
            density_override = Some(
                line.split(':').nth(1)
                    .unwrap_or("0").trim()
                    .parse().unwrap_or(0)
            );
        }
    }
    if density_physical == 0 {
        let prop = run_adb_cmd_with_device(&adb, s, &["shell", "getprop", "ro.sf.lcd_density"]).unwrap_or_default();
        density_physical = prop.trim().parse().unwrap_or(0);
    }
    let density_current = density_override.unwrap_or(density_physical);

    Ok(ToggleStates {
        wifi: parse_bool(&wifi_output),
        data: parse_bool(&data_output),
        airplane: parse_bool(&airplane_output),
        bluetooth: parse_bool(&bt_output),
        show_taps: parse_bool(&taps_output),
        layout_bounds: parse_bool(&layout_output),
        stay_awake: parse_bool(&awake_output),
        brightness: brightness_val,
        density: density_current,
        density_override: density_override,
        density_physical: density_physical,
    })
}

#[tauri::command]
pub fn adb_rotate(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let _ = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "settings", "put", "system", "accelerometer_rotation", "0"]);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "input", "keyevent", "276"])
}

// ─── Density Commands ───────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct DensityInfo {
    pub physical: i32,
    pub override_density: Option<i32>,
    pub current: i32,
}

#[tauri::command]
pub fn adb_get_density(state: State<AppState>) -> Result<DensityInfo, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    let output = run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "wm", "density"])?;

    let mut physical: i32 = 0;
    let mut override_density: Option<i32> = None;

    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("Physical density:") {
            physical = line.split(':').nth(1)
                .unwrap_or("0").trim()
                .parse().unwrap_or(0);
        } else if line.starts_with("Override density:") {
            override_density = Some(
                line.split(':').nth(1)
                    .unwrap_or("0").trim()
                    .parse().unwrap_or(0)
            );
        }
    }

    // R1 Mitigation: fallback to getprop if wm density returns nothing
    if physical == 0 {
        let prop = run_adb_cmd_with_device(&adb, serial.as_deref(),
            &["shell", "getprop", "ro.sf.lcd_density"])
            .unwrap_or_default();
        physical = prop.trim().parse().unwrap_or(0);
    }

    let current = override_density.unwrap_or(physical);

    Ok(DensityInfo {
        physical,
        override_density,
        current,
    })
}

#[tauri::command]
pub fn adb_set_density(state: State<AppState>, value: i32) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    let clamped = value.clamp(120, 640);
    let val_str = clamped.to_string();
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "wm", "density", &val_str])
}

#[tauri::command]
pub fn adb_reset_density(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);
    run_adb_cmd_with_device(&adb, serial.as_deref(), &["shell", "wm", "density", "reset"])
}


// ─── Clipboard Sync Commands ──────────────────────────────────

fn chrono_or_simple_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

use std::io::BufRead;
use regex::Regex;
use std::sync::OnceLock;
use crate::LogEntry;

// ─── Logcat Parser ─────────────────────────────────────────

static LOGCAT_RE: OnceLock<Regex> = OnceLock::new();

fn get_logcat_re() -> &'static Regex {
    LOGCAT_RE.get_or_init(|| {
        Regex::new(
            r"^(\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\.\d{3})\s+(\d+)\s+(\d+)\s+([VDIWEF])\s+(.+?):\s*(.*)$"
        ).expect("hardcoded regex must compile")
    })
}

enum LogLine {
    Entry(LogEntry),
    Separator(String),
    Raw(String),
    Empty,
}

fn parse_logcat_line(line: &str) -> LogLine {
    if line.starts_with("------") {
        return LogLine::Separator(line.to_string());
    }
    if line.trim().is_empty() {
        return LogLine::Empty;
    }
    if let Some(caps) = get_logcat_re().captures(line) {
        return LogLine::Entry(LogEntry {
            id: 0,
            timestamp: caps[1].to_string(),
            pid: caps[2].to_string(),
            tid: caps[3].to_string(),
            level: caps[4].chars().next().unwrap_or('V'),
            tag: caps[5].trim().to_string(),
            message: caps[6].to_string(),
            raw: None,
        });
    }
    LogLine::Raw(line.to_string())
}

// ─── Logcat Commands ───────────────────────────────────────

#[tauri::command]
pub fn adb_start_logcat(
    app: tauri::AppHandle,
    state: State<AppState>,
    channel: tauri::ipc::Channel<LogEntry>,
) -> Result<(), String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state)
        .ok_or("No device connected")?;

    {
        let proc = lock_state!(state.logcat_process);
        if proc.is_some() {
            return Err("Logcat already running".to_string());
        }
    }

    // Clean up any stale thread handle from a previous run.
    // We intentionally do NOT join() — joining a stuck reader thread would
    // block this command and freeze the UI. Dropping the handle detaches it.
    {
        let mut thread = lock_state!(state.logcat_thread);
        if let Some(handle) = thread.take() {
            drop(handle);
        }
    }

    state.logcat_cancel.store(false, Ordering::SeqCst);

    let args_owned: Vec<String> = vec![
        "-s".to_string(), serial.clone(),
        "logcat".to_string(),
        "-v".to_string(), "threadtime".to_string(),
    ];

    #[cfg(windows)]
    let mut child = Command::new(&adb)
        .args(&args_owned)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("Failed to spawn adb logcat: {}", e))?;

    #[cfg(not(windows))]
    let mut child = Command::new(&adb)
        .args(&args_owned)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn adb logcat: {}", e))?;

    let pid = child.id();
    {
        let mut proc = lock_state!(state.logcat_process);
        *proc = Some(pid);
    }

    let cancel = state.logcat_cancel.clone();
    let logcat_process = state.logcat_process.clone();
    let app_handle = app.clone();

    let handle = std::thread::spawn(move || {
        // Spawn a stderr reader thread to capture any error output
        let stderr_pipe = child.stderr.take();
        let stderr_cancel = cancel.clone();
        let stderr_thread = stderr_pipe.map(|pipe| {
            std::thread::spawn(move || {
                let reader = std::io::BufReader::new(pipe);
                let mut output = String::new();
                for line in reader.lines() {
                    if stderr_cancel.load(Ordering::SeqCst) { break; }
                    match line {
                        Ok(l) => {
                            output.push_str(&l);
                            output.push('\n');
                        }
                        Err(_) => break,
                    }
                }
                output
            })
        });

        let stdout = child.stdout.take();
        if let Some(out) = stdout {
            let reader = std::io::BufReader::new(out);
            let mut last_entry: Option<LogEntry> = None;
            let mut monotonic_id: u64 = 0;

            for line_result in reader.lines() {
                if cancel.load(Ordering::SeqCst) { break; }
                let line = line_result.unwrap_or_default();

                match parse_logcat_line(&line) {
                    LogLine::Entry(mut entry) => {
                        if let Some(e) = last_entry.take() {
                            let _ = channel.send(e);
                        }
                        monotonic_id += 1;
                        entry.id = monotonic_id;
                        last_entry = Some(entry);
                    }
                    LogLine::Raw(text) => {
                        if let Some(ref mut e) = last_entry {
                            e.message.push('\n');
                            e.message.push_str(&text);
                        } else {
                            monotonic_id += 1;
                            let _ = channel.send(LogEntry {
                                id: monotonic_id,
                                timestamp: String::new(),
                                pid: String::new(),
                                tid: String::new(),
                                level: 'V',
                                tag: "UNKNOWN".to_string(),
                                message: text.clone(),
                                raw: Some(text),
                            });
                        }
                    }
                    LogLine::Separator(text) => {
                        if let Some(e) = last_entry.take() {
                            let _ = channel.send(e);
                        }
                        monotonic_id += 1;
                        let _ = channel.send(LogEntry {
                            id: monotonic_id,
                            timestamp: String::new(),
                            pid: String::new(),
                            tid: String::new(),
                            level: 'V',
                            tag: "SYSTEM".to_string(),
                            message: text,
                            raw: None,
                        });
                    }
                    LogLine::Empty => {}
                }
            }

            if let Some(e) = last_entry {
                let _ = channel.send(e);
            }
        }

        // ── Process exited – clean up and notify frontend ────────────────
        let exit_status = child.wait().ok();
        let was_cancelled = cancel.load(Ordering::SeqCst);

        // Collect stderr output from the reader thread
        let stderr_output = stderr_thread
            .and_then(|h| h.join().ok())
            .unwrap_or_default();

        // Clear the stored PID so a new start won't hit "Logcat already running"
        {
            let mut proc = logcat_process.lock().unwrap();
            *proc = None;
        }

        // Emit event to frontend so it can update UI state
        let _ = app_handle.emit(
            "logcat-exited",
            serde_json::json!({
                "cancelled": was_cancelled,
                "exitCode": exit_status.and_then(|s| s.code()),
                "stderr": stderr_output.trim(),
            }),
        );
    });

    {
        let mut thread = lock_state!(state.logcat_thread);
        *thread = Some(handle);
    }

    Ok(())
}

#[tauri::command]
pub fn adb_stop_logcat(state: State<AppState>) -> Result<(), String> {
    state.logcat_cancel.store(true, Ordering::SeqCst);
    let pid = {
        let mut proc = lock_state!(state.logcat_process);
        proc.take()
    };
    if let Some(pid) = pid {
        kill_process_tree(pid);
    }

    // Do NOT block on join() — if kill_process_tree failed to terminate the
    // process, the reader thread would hang forever and freeze the UI.
    // Dropping the JoinHandle detaches the thread; it will exit on its own
    // once the process dies and stdout reaches EOF.
    let _ = {
        let mut thread = lock_state!(state.logcat_thread);
        thread.take()
    };

    Ok(())
}

#[tauri::command]
pub fn adb_clear_logcat_buffer(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state)
        .ok_or("No device connected")?;

    let result = run_adb_cmd_with_device(&adb, Some(&serial), &["logcat", "-c"]
    );

    if let Err(ref e) = result {
        if e.contains("permission") || e.contains("failed to clear") || e.contains("denied") {
            return run_adb_cmd_with_device(
                &adb, Some(&serial), &["logcat", "-b", "all", "-c"]
            );
        }
    }
    result
}

// ─── Active App Filter (Logcat) ────────────────────────────

#[tauri::command]
fn get_foreground_package_via_window(adb: &str, serial: &str) -> Result<String, String> {
    let output = run_adb_cmd_with_device(adb, Some(serial), &["shell", "dumpsys", "window", "windows"])?;

    let package = output
        .lines()
        .find(|l| l.contains("mCurrentFocus") || l.contains("mFocusedWindow"))
        .and_then(|line| {
            line.split_whitespace()
                .filter(|token| token.contains('/'))
                .last()
                .map(|token| token.split('/').next().unwrap_or("").trim())
        })
        .unwrap_or("")
        .to_string();

    Ok(package)
}

fn get_foreground_package_via_activity(adb: &str, serial: &str) -> Result<String, String> {
    let output = run_adb_cmd_with_device(adb, Some(serial), &["shell", "dumpsys", "activity", "activities"])?;

    let package = output
        .lines()
        .find(|l| l.contains("mResumedActivity") || l.contains("topResumedActivity"))
        .and_then(|line| {
            line.split_whitespace()
                .filter(|token| token.contains('/'))
                .last()
                .map(|token| token.split('/').next().unwrap_or("").trim().trim_end_matches('}'))
        })
        .unwrap_or("")
        .to_string();

    Ok(package)
}

#[tauri::command]
pub fn adb_get_foreground_package(state: State<AppState>) -> Result<String, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state)
        .ok_or("No device connected")?;

    if let Ok(pkg) = get_foreground_package_via_window(&adb, &serial) {
        if !pkg.is_empty() {
            return Ok(pkg);
        }
    }

    if let Ok(pkg) = get_foreground_package_via_activity(&adb, &serial) {
        if !pkg.is_empty() {
            return Ok(pkg);
        }
    }

    Err("Could not determine foreground package".to_string())
}

#[tauri::command]
pub fn adb_get_pids_for_package(state: State<AppState>, package: String) -> Result<Vec<String>, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state)
        .ok_or("No device connected")?;

    // pidof returns space-separated PIDs
    let output = run_adb_cmd_with_device(
        &adb,
        Some(&serial),
        &["shell", "pidof", &package],
    )
    .unwrap_or_default()
    .trim()
    .to_string();

    if output.is_empty() {
        return Ok(vec![]);
    }

    let pids: Vec<String> = output
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    Ok(pids)
}

// ─── Utility ─────────────────────────────────────────────

#[tauri::command]
pub fn write_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

// ─── App Manager Commands ──────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppInfo {
    pub package_name: String,
    pub label: String,
    pub version_name: String,
    pub version_code: i64,
    pub is_system: bool,
    pub is_enabled: bool,
    pub is_updated_system: bool,
    pub code_path: String,
    pub data_dir: String,
    pub apk_size: Option<i64>,
    pub first_install_time: Option<String>,
    pub last_update_time: Option<String>,
}

/// Derive a fallback label from the last segment of a package name.
fn derive_label(package_name: &str) -> String {
    package_name
        .rsplit('.')
        .next()
        .unwrap_or(package_name)
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_uppercase().next().unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

fn parse_pm_list(output: &str, filter: &str) -> Vec<AppInfo> {
    let mut apps = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Strip "package:" prefix
        let stripped = if let Some(rest) = line.strip_prefix("package:") {
            rest
        } else {
            continue;
        };

        let (path, package_name) = if let Some(eq_pos) = stripped.rfind('=') {
            // Format: /path/to/base.apk=com.example.app
            let pkg = stripped[eq_pos + 1..].to_string();
            let path = stripped[..eq_pos].to_string();
            (path, pkg)
        } else {
            // Format: com.example.app (no path)
            (String::new(), stripped.to_string())
        };

        if package_name.is_empty() {
            continue;
        }

        // Derive label from last segment of package name as fallback
        let label = derive_label(&package_name);

        let is_enabled = filter != "disabled";

        apps.push(AppInfo {
            package_name,
            label,
            version_name: String::new(),
            version_code: 0,
            is_system: false, // will be set by caller if needed
            is_enabled,
            is_updated_system: false,
            code_path: path,
            data_dir: String::new(),
            apk_size: None,
            first_install_time: None,
            last_update_time: None,
        });
    }
    apps
}

/// Resolve real app labels by running an embedded DEX on the device via
/// `app_process`. Uses reflection to call `PackageManager.getApplicationLabel()`
/// which correctly resolves `@string/app_name` resources.
///
/// Flow:
///   1. Write embedded DEX bytes to a local temp file.
///   2. `adb push` it to `/data/local/tmp/adbph_labels.dex`.
///   3. Execute `CLASSPATH=... app_process / AppLabels`.
///   4. Parse stdout lines as `package_name|label`.
///   5. Best-effort cleanup of remote and local temp files.
fn resolve_labels_via_dex(
    adb: &str,
    serial: Option<&str>,
    package_filter: Option<&str>,
) -> Result<std::collections::HashMap<String, String>, String> {
    use std::io::Write;

    // 1. Write embedded DEX to local temp
    let temp_dir = std::env::temp_dir();
    let local_dex = temp_dir.join("adbph_labels.dex");
    {
        let mut file = std::fs::File::create(&local_dex)
            .map_err(|e| format!("Failed to create temp dex: {}", e))?;
        file.write_all(APP_LABELS_DEX)
            .map_err(|e| format!("Failed to write temp dex: {}", e))?;
    }

    // 2. Push to device
    let local_path = local_dex.to_string_lossy();
    let push_result = run_adb_cmd_with_device_timed(
        adb,
        serial,
        &["push", &local_path, DEX_REMOTE_PATH],
        30,
    );
    if let Err(e) = push_result {
        let _ = std::fs::remove_file(&local_dex);
        return Err(format!("Failed to push dex: {}", e));
    }

    // 3. Run label resolver (batch or single-package)
    let shell_cmd = if let Some(pkg) = package_filter {
        format!(
            "CLASSPATH={} app_process / AppLabels {}",
            DEX_REMOTE_PATH, pkg
        )
    } else {
        format!(
            "CLASSPATH={} app_process / AppLabels",
            DEX_REMOTE_PATH
        )
    };
    let run_result = run_adb_cmd_with_device_timed(
        adb,
        serial,
        &["shell", &shell_cmd],
        60,
    );

    // 4. Parse output
    let mut labels = std::collections::HashMap::new();
    match run_result {
        Ok(output) => {
            for line in output.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("ERROR:") {
                    continue;
                }
                if let Some(pipe_idx) = line.find('|') {
                    let pkg = line[..pipe_idx].to_string();
                    let label = line[pipe_idx + 1..].to_string();
                    if !pkg.is_empty() && !label.is_empty() {
                        labels.insert(pkg, label);
                    }
                }
            }
        }
        Err(e) => {
            // Even on run failure, attempt cleanup then return error
            let _ = run_adb_cmd_with_device_timed(
                adb,
                serial,
                &["shell", &format!("rm -f {}", DEX_REMOTE_PATH)],
                5,
            );
            let _ = std::fs::remove_file(&local_dex);
            return Err(format!("Label resolver failed: {}", e));
        }
    }

    // 5. Best-effort cleanup
    let _ = run_adb_cmd_with_device_timed(
        adb,
        serial,
        &["shell", &format!("rm -f {}", DEX_REMOTE_PATH)],
        5,
    );
    let _ = std::fs::remove_file(&local_dex);

    Ok(labels)
}

fn parse_package_names(output: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("package:") {
            // Strip path prefix if present (e.g. "package:/data/app/...=com.foo")
            let name = if let Some(eq_pos) = rest.rfind('=') {
                rest[eq_pos + 1..].to_string()
            } else {
                rest.to_string()
            };
            if !name.is_empty() {
                names.push(name);
            }
        }
    }
    names
}

fn parse_dumpsys_package(output: &str, package: &str) -> Result<AppInfo, String> {
    let mut version_name = String::new();
    let mut version_code: i64 = 0;
    let mut is_system = false;
    let mut is_enabled = true;
    let mut is_updated_system = false;
    let mut code_path = String::new();
    let mut data_dir = String::new();
    let mut label = String::new();
    let mut first_install_time: Option<String> = None;
    let mut last_update_time: Option<String> = None;

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("versionName=") {
            version_name = trimmed
                .strip_prefix("versionName=")
                .unwrap_or("")
                .to_string();
        } else if trimmed.starts_with("versionCode=") {
            // versionCode=123 or versionCode=123 minSdk=...
            let rest = trimmed.strip_prefix("versionCode=").unwrap_or("");
            version_code = rest
                .split_whitespace()
                .next()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
        } else if trimmed.starts_with("codePath=") {
            code_path = trimmed
                .strip_prefix("codePath=")
                .unwrap_or("")
                .to_string();
        } else if trimmed.starts_with("dataDir=") {
            data_dir = trimmed
                .strip_prefix("dataDir=")
                .unwrap_or("")
                .to_string();
        } else if trimmed.starts_with("flags=[") || trimmed.starts_with("flags=") {
            let flags_str = if trimmed.starts_with("flags=[") {
                // flags=[ SYSTEM ... ]
                let end = trimmed.find(']').unwrap_or(trimmed.len());
                &trimmed[7..end]
            } else {
                // flags=0x...
                let hex = trimmed.strip_prefix("flags=").unwrap_or("");
                let val: i64 = i64::from_str_radix(hex.trim_start_matches("0x"), 16).unwrap_or(0);
                if val & 0x00000001 != 0 {
                    is_system = true;
                }
                if val & 0x00000080 != 0 {
                    is_updated_system = true;
                }
                if val & 0x00000010 != 0 {
                    is_enabled = false;
                }
                continue;
            };
            // Parse text flags like "SYSTEM HAS_CODE"
            is_system |= flags_str.contains("SYSTEM");
            is_updated_system |= flags_str.contains("UPDATED_SYSTEM_APP");
            if flags_str.contains("DISABLED") && !flags_str.contains("ENABLED") {
                is_enabled = false;
            }
        } else if trimmed.starts_with("nonLocalizedLabel=") {
            let lbl = trimmed
                .strip_prefix("nonLocalizedLabel=")
                .unwrap_or("")
                .to_string();
            // Only use if non-empty, not "null", and not numeric placeholder
            if !lbl.is_empty()
                && lbl != "null"
                && !lbl
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
            {
                label = lbl;
            }
        } else if trimmed.starts_with("firstInstallTime=") {
            // firstInstallTime=2024-03-15 10:30:00
            let raw = trimmed
                .strip_prefix("firstInstallTime=")
                .unwrap_or("")
                .trim()
                .to_string();
            if !raw.is_empty() && raw != "0" {
                // Keep only the date portion (YYYY-MM-DD)
                first_install_time = Some(raw.split_whitespace().next().unwrap_or(&raw).to_string());
            }
        } else if trimmed.starts_with("lastUpdateTime=") {
            // lastUpdateTime=2026-06-01 14:22:33
            let raw = trimmed
                .strip_prefix("lastUpdateTime=")
                .unwrap_or("")
                .trim()
                .to_string();
            if !raw.is_empty() && raw != "0" {
                last_update_time = Some(raw.split_whitespace().next().unwrap_or(&raw).to_string());
            }
        }
    }

    // Fallback label from package name
    if label.is_empty() {
        label = derive_label(package);
    }

    Ok(AppInfo {
        package_name: package.to_string(),
        label,
        version_name,
        version_code,
        is_system,
        is_enabled,
        is_updated_system,
        code_path,
        data_dir,
        apk_size: None,
        first_install_time,
        last_update_time,
    })
}

static ICONS_FETCH_RUNNING: AtomicBool = AtomicBool::new(false);

/// Run the AppIcons DEX on device to extract icons via ZIP reading.
/// Returns HashMap<package, Vec<u8>> of PNG bytes.
fn resolve_icons_via_zip_dex(
    adb: &str,
    serial: Option<&str>,
    packages: &[(&str, &str)], // (pkg, apk_path)
) -> Result<std::collections::HashMap<String, Vec<u8>>, String> {
    use std::io::Write;

    if packages.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    // 1. Push DEX to device
    let temp_dir = std::env::temp_dir();
    let local_dex = temp_dir.join("adbph_icons_v2.dex");
    {
        let mut file = std::fs::File::create(&local_dex)
            .map_err(|e| format!("Failed to create temp dex: {}", e))?;
        file.write_all(APP_ICONS_DEX)
            .map_err(|e| format!("Failed to write temp dex: {}", e))?;
    }
    let local_path = local_dex.to_string_lossy();
    run_adb_cmd_with_device_timed(adb, serial, &["push", &local_path, ICONS_DEX_REMOTE], 30)
        .map_err(|e| {
            let _ = std::fs::remove_file(&local_dex);
            format!("Failed to push icons dex: {}", e)
        })?;

    // 2. Build argument: "pkg1=apk1,pkg2=apk2,..."
    let pkg_arg: String = packages
        .iter()
        .map(|(pkg, apk)| format!("{}={}", pkg, apk))
        .collect::<Vec<_>>()
        .join(",");

    let shell_cmd = format!(
        "CLASSPATH={} app_process / AppIcons {} 2>&1",
        ICONS_DEX_REMOTE, pkg_arg
    );

    // 3. Run DEX (quiet)
    let output = run_adb_cmd_with_device_timed(
        adb, serial,
        &["shell", &shell_cmd],
        ICONS_DEX_TIMEOUT_SECS,
    );
    // Log errors too — stderr is mixed into the result on failure
    let dex_output = match &output {
        Ok(o) => o.clone(),
        Err(_) => String::new()
    };

    // 4. Parse output: pkg|base64
    let mut result = std::collections::HashMap::new();
    for line in dex_output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("ERROR:") || line.starts_with("SKIP:") {
            continue;
        }
        // Split on first '|': pkg|base64
        if let Some(pipe_idx) = line.find('|') {
            let pkg = line[..pipe_idx].to_string();
            let b64 = line[pipe_idx + 1..].to_string();
            if pkg.is_empty() || b64.is_empty() {
                continue;
            }
            match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &b64) {
                Ok(png_bytes) => {
                    result.insert(pkg, png_bytes);
                }
                Err(_) => continue,
            }
        }
    }

    // 5. Cleanup
    let _ = run_adb_cmd_with_device_timed(adb, serial, &["shell", &format!("rm -f {}", ICONS_DEX_REMOTE)], 5);
    let _ = std::fs::remove_file(&local_dex);

    Ok(result)
}

/// Brief package info needed for icon extraction (avoids pulling full AppInfo).
#[derive(Deserialize)]
pub struct PackageIconRequest {
    package_name: String,
    code_path: String,
    version_code: i64,
}

#[tauri::command]
pub fn adb_fetch_icons(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    packages: Vec<PackageIconRequest>,
) -> Result<(), String> {
    use crate::icons;
    use std::time::Instant;

    // Concurrency guard
    if ICONS_FETCH_RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return Err("Icon fetch already in progress".into());
    }

    // Clone everything the background thread needs
    let adb = get_adb_path(&state);
    let serial = match get_device_serial(&state) {
        Some(s) => s,
        None => {
            ICONS_FETCH_RUNNING.store(false, Ordering::SeqCst);
            return Err("No device connected".into());
        }
    };

    let data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Cannot get app data dir: {}", e))?;
    let serial_safe = serial.replace(':', "_").replace('/', "_").replace('\\', "_");
    let cache_dir = data_dir.join("adbph_icons").join(&serial_safe);

    // Spawn background thread — UI stays responsive, icons arrive via events
    let app = app_handle.clone();
    std::thread::spawn(move || {
        // Ensure concurrency guard is ALWAYS released, even on panic
        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) {
                ICONS_FETCH_RUNNING.store(false, Ordering::SeqCst);
            }
        }
        let _guard = Guard;

        let start = Instant::now();
        let package_names: Vec<String> = packages.iter().map(|p| p.package_name.clone()).collect();

        // 1. Emit cached icons immediately
        let mut missing: Vec<&PackageIconRequest> = Vec::new();
        let mut cached_count = 0u32;

        for pkg in &packages {
            if let Some(data) = icons::get_cached(&cache_dir, &pkg.package_name, pkg.version_code) {
                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
                let _ = app.emit("icon-ready", serde_json::json!({
                    "package": &pkg.package_name,
                    "base64": b64,
                }));
                cached_count += 1;
            } else {
                missing.push(pkg);
            }
        }

        eprintln!(
            "[icons] Fetch — total: {}, cached: {}",
            packages.len(), cached_count
        );

        // 2. Extract missing icons via on-device ZIP-reading DEX (single call!)
        let mut success = 0u32;
        let mut failed = 0u32;

        if !missing.is_empty() {
            // Build pkg=apk pairs for the DEX
            let dex_packages: Vec<(&str, &str)> = missing
                .iter()
                .filter(|p| !p.code_path.is_empty())
                .map(|p| (p.package_name.as_str(), p.code_path.as_str()))
                .collect();

            match resolve_icons_via_zip_dex(&adb, Some(&serial), &dex_packages) {
                Ok(extracted) => {
                    for pkg_req in &missing {
                        if pkg_req.code_path.is_empty() {
                            failed += 1;
                            let _ = app.emit("icon-ready", serde_json::json!({
                                "package": &pkg_req.package_name,
                                "base64": null,
                            }));
                            continue;
                        }
                        if let Some(png_data) = extracted.get(&pkg_req.package_name) {
                            // Save to cache
                            icons::save_to_cache(
                                &cache_dir,
                                &pkg_req.package_name,
                                pkg_req.version_code,
                                png_data,
                                false,
                            );
                            let b64 = base64::Engine::encode(
                                &base64::engine::general_purpose::STANDARD,
                                png_data,
                            );
                            let _ = app.emit("icon-ready", serde_json::json!({
                                "package": &pkg_req.package_name,
                                "base64": b64,
                            }));
                            success += 1;
                        } else {
                            failed += 1;
                            let _ = app.emit("icon-ready", serde_json::json!({
                                "package": &pkg_req.package_name,
                                "base64": null,
                            }));
                        }
                    }
                }
                Err(_e) => {
                    failed = missing.len() as u32;
                    for pkg in &missing {
                        let _ = app.emit("icon-ready", serde_json::json!({
                            "package": &pkg.package_name,
                            "base64": null,
                        }));
                    }
                }
            }
        }

        // 3. Cleanup
        icons::cleanup_stale_cache(&cache_dir, &package_names);
        icons::evict_lru(&cache_dir, 50 * 1024 * 1024);

        // 4. Emit completion event
        let elapsed = start.elapsed();
        eprintln!(
            "[icons] Fetch complete — total: {}, cached: {}, extracted: {}, failed: {}, duration: {:.1}s",
            packages.len(), cached_count, success, failed, elapsed.as_secs_f32()
        );
        let _ = app.emit("icons-fetch-complete", serde_json::json!({
            "total": packages.len(),
            "cached": cached_count,
            "extracted": success,
            "failed": failed,
            "duration_secs": elapsed.as_secs_f32(),
        }));

        // Guard (_guard) releases ICONS_FETCH_RUNNING on drop
    });

    // Return immediately — work continues in background thread
    Ok(())
}

#[tauri::command]
pub fn adb_list_apps(state: State<AppState>, filter: String) -> Result<Vec<AppInfo>, String> {
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state).ok_or("No device connected")?;

    // Build pm list packages args based on filter
    let mut args: Vec<String> = vec![
        "shell".into(),
        "pm".into(),
        "list".into(),
        "packages".into(),
        "-f".into(),
    ];
    match filter.as_str() {
        "third_party" => args.push("-3".into()),
        "system" => args.push("-s".into()),
        "disabled" => args.push("-d".into()),
        _ => {} // "all" — no filter flag
    }

    let output = run_adb_cmd_with_device_timed(
        &adb,
        Some(&serial),
        &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        30,
    )?;

    // For "all" filter, also fetch system packages list for cross-referencing
    let system_packages: std::collections::HashSet<String> = if filter == "all" {
        let sys_output = run_adb_cmd_with_device_timed(
            &adb,
            Some(&serial),
            &["shell", "pm", "list", "packages", "-s"],
            30,
        )?;
        parse_package_names(&sys_output)
            .into_iter()
            .collect()
    } else {
        std::collections::HashSet::new()
    };

    let mut apps = parse_pm_list(&output, &filter);

    // Resolve real app labels via on-device DEX (PackageManager.getApplicationLabel)
    let real_labels = resolve_labels_via_dex(
        &adb,
        Some(&serial),
        None,
    )
    .unwrap_or_else(|e| {
        eprintln!("[adb] resolve_labels_via_dex failed: {}, using fallback", e);
        std::collections::HashMap::new()
    });
    for app in &mut apps {
        if let Some(label) = real_labels.get(&app.package_name) {
            app.label = label.clone();
        }
    }

    // Set is_system based on filter logic
    match filter.as_str() {
        "system" => {
            for app in &mut apps {
                app.is_system = true;
            }
        }
        "third_party" => {
            for app in &mut apps {
                app.is_system = false;
            }
        }
        "disabled" => {
            for app in &mut apps {
                app.is_system = true; // disabled apps are typically system
                app.is_enabled = false;
            }
        }
        _ => {
            // "all" — cross-reference with system list
            for app in &mut apps {
                app.is_system = system_packages.contains(&app.package_name);
            }
        }
    }

    Ok(apps)
}

#[tauri::command]
pub fn adb_app_detail(state: State<AppState>, package: String) -> Result<AppInfo, String> {
    let _serial = get_device_serial(&state).ok_or("No device connected")?;
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    let output = run_adb_cmd_with_device(
        &adb,
        serial.as_deref(),
        &["shell", "dumpsys", "package", &package],
    )?;

    let mut info = parse_dumpsys_package(&output, &package)?;

    // Get APK size via stat (non-fatal if it fails)
    if !info.code_path.is_empty() {
        if let Ok(size_output) = run_adb_cmd_with_device(
            &adb,
            serial.as_deref(),
            &["shell", "stat", "-c", "%s", &info.code_path],
        ) {
            info.apk_size = size_output.trim().parse().ok();
        }
    }

    Ok(info)
}

#[tauri::command]
pub async fn adb_install_apk(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let _serial = get_device_serial(&state).ok_or("No device connected")?;
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    run_adb_cmd_with_device_timed(
        &adb,
        serial.as_deref(),
        &["install", "-r", &path],
        120,
    )
}

#[tauri::command]
pub fn adb_uninstall_app(
    state: State<AppState>,
    package: String,
    is_system: bool,
) -> Result<String, String> {
    let _serial = get_device_serial(&state).ok_or("No device connected")?;
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    if is_system {
        // For system apps, uninstall for current user only (can be re-enabled)
        run_adb_cmd_with_device(
            &adb,
            serial.as_deref(),
            &["shell", "pm", "uninstall", "-k", "--user", "0", &package],
        )
    } else {
        run_adb_cmd_with_device(
            &adb,
            serial.as_deref(),
            &["shell", "pm", "uninstall", &package],
        )
    }
}

#[tauri::command]
pub fn adb_clear_app(state: State<AppState>, package: String) -> Result<String, String> {
    let _serial = get_device_serial(&state).ok_or("No device connected")?;
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    run_adb_cmd_with_device(
        &adb,
        serial.as_deref(),
        &["shell", "pm", "clear", &package],
    )
}

#[tauri::command]
pub fn adb_force_stop_app(state: State<AppState>, package: String) -> Result<String, String> {
    let _serial = get_device_serial(&state).ok_or("No device connected")?;
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    run_adb_cmd_with_device(
        &adb,
        serial.as_deref(),
        &["shell", "am", "force-stop", &package],
    )
}

#[tauri::command]
pub fn adb_enable_app(state: State<AppState>, package: String) -> Result<String, String> {
    let _serial = get_device_serial(&state).ok_or("No device connected")?;
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    run_adb_cmd_with_device(
        &adb,
        serial.as_deref(),
        &["shell", "pm", "enable", &package],
    )
}

#[tauri::command]
pub fn adb_disable_app(state: State<AppState>, package: String) -> Result<String, String> {
    let _serial = get_device_serial(&state).ok_or("No device connected")?;
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    run_adb_cmd_with_device(
        &adb,
        serial.as_deref(),
        &["shell", "pm", "disable-user", "--user", "0", &package],
    )
}

#[tauri::command]
pub fn adb_open_app(state: State<AppState>, package: String) -> Result<String, String> {
    let _serial = get_device_serial(&state).ok_or("No device connected")?;
    let adb = get_adb_path(&state);
    let serial = get_device_serial(&state);

    // Use monkey command — simplest way to launch an app's main activity
    // without needing to resolve the launchable activity explicitly.
    // Non-fatal if the app has no launcher intent.
    run_adb_cmd_with_device(
        &adb,
        serial.as_deref(),
        &["shell", "monkey", "-p", &package, "-c", "android.intent.category.LAUNCHER", "1"],
    )
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

// ─── Screenshot Gallery Helpers ──────────────────────────────

/// Read a file and return its contents as a base64-encoded string.
fn read_file_base64(path: &str) -> Result<String, String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("Cannot open file: {}", e))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| format!("Cannot read file: {}", e))?;
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

// ─── Screenshot Gallery Structs ──────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenshotFile {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_iso: String,
    pub dimensions: Option<ImageDimensions>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenshotListResult {
    pub files: Vec<ScreenshotFile>,
    pub total_count: usize,
    pub truncated: bool,
}

// ─── Screenshot Gallery Helpers ──────────────────────────────

/// Read image dimensions from a file (PNG, JPEG, WebP supported).
/// Uses the `image` crate's `image_dimensions()` which reads only headers.
fn read_image_dimensions(path: &std::path::PathBuf) -> Option<ImageDimensions> {
    match image::image_dimensions(path) {
        Ok((width, height)) => {
            if width == 0 || height == 0 {
                None
            } else {
                Some(ImageDimensions { width, height })
            }
        }
        Err(_) => None,
    }
}

/// Convert SystemTime to ISO 8601 string without external crates.
/// Format: "YYYY-MM-DDTHH:MM:SSZ"
fn system_time_to_iso8601(t: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let duration = t.duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_secs = duration.as_secs();

    // Days since epoch → year/month/day calculation
    let mut days = total_secs / 86400;
    let time_of_day = total_secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year (starting from 1970)
    let mut year: i64 = 1970;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year as u64 {
            break;
        }
        days -= days_in_year as u64;
        year += 1;
    }

    // Calculate month and day
    let months_days: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month: u64 = 1;
    for md in months_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    let day = days + 1;

    format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

// ─── Screenshot Gallery Commands ─────────────────────────────

#[tauri::command]
pub fn adb_list_screenshots(dir_path: Option<String>) -> Result<ScreenshotListResult, String> {
    let dir = dir_path
        .filter(|p| !p.is_empty())
        .unwrap_or_else(dirs_or_fallback_screenshots);

    let dir_path_buf = std::path::PathBuf::from(&dir);
    if !dir_path_buf.exists() || !dir_path_buf.is_dir() {
        return Ok(ScreenshotListResult {
            files: vec![],
            total_count: 0,
            truncated: false,
        });
    }

    let mut files: Vec<ScreenshotFile> = Vec::new();
    let mut total_count: usize = 0;
    const MAX_FILES: usize = 500;

    if let Ok(entries) = std::fs::read_dir(&dir_path_buf) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();
            if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp") {
                continue;
            }

            total_count += 1;
            if files.len() >= MAX_FILES {
                continue;
            }

            let metadata = match std::fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let created = metadata
                .modified()
                .or_else(|_| metadata.created())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let dimensions = read_image_dimensions(&path);

            files.push(ScreenshotFile {
                filename,
                path: path.to_string_lossy().to_string(),
                size_bytes: metadata.len(),
                created_iso: system_time_to_iso8601(created),
                dimensions,
            });
        }
    }

    // Sort newest first by created_iso (string comparison works for ISO 8601)
    files.sort_by(|a, b| b.created_iso.cmp(&a.created_iso));

    Ok(ScreenshotListResult {
        files,
        total_count,
        truncated: total_count > MAX_FILES,
    })
}

#[tauri::command]
pub fn adb_delete_screenshot(path: String, save_dir: Option<String>) -> Result<(), String> {
    let path_buf = std::path::PathBuf::from(&path);
    let canonical = path_buf
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;

    let save_dir_path = save_dir
        .filter(|p| !p.is_empty())
        .unwrap_or_else(dirs_or_fallback_screenshots);
    let save_dir_canonical = std::path::PathBuf::from(&save_dir_path)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(&save_dir_path));

    // Path traversal guard
    if !canonical.starts_with(&save_dir_canonical) {
        return Err("Access denied: file is outside screenshot directory".to_string());
    }
    if !canonical.is_file() {
        return Err("Not a file".to_string());
    }

    std::fs::remove_file(&canonical).map_err(|e| format!("Failed to delete: {}", e))
}

#[tauri::command]
pub fn adb_get_file_info(path: String) -> Result<ScreenshotFile, String> {
    let path_buf = std::path::PathBuf::from(&path);
    if !path_buf.is_file() {
        return Err("File not found".to_string());
    }

    let metadata = std::fs::metadata(&path_buf)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    let created = metadata
        .modified()
        .or_else(|_| metadata.created())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

    let filename = path_buf
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let dimensions = read_image_dimensions(&path_buf);

    Ok(ScreenshotFile {
        filename,
        path: path_buf.to_string_lossy().to_string(),
        size_bytes: metadata.len(),
        created_iso: system_time_to_iso8601(created),
        dimensions,
    })
}

#[tauri::command]
pub fn adb_read_file_base64(path: String) -> Result<String, String> {
    read_file_base64(&path)
}

/// Generate a thumbnail for a screenshot file.
/// Returns base64-encoded JPEG or None if the image cannot be loaded.
/// Includes path traversal guard (NFR-2).
fn generate_thumbnail(path: &std::path::PathBuf) -> Result<Option<String>, String> {
    // Path traversal guard — same pattern as adb_delete_screenshot
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;

    let save_dir = dirs_or_fallback_screenshots();
    let save_dir_canonical = std::path::PathBuf::from(&save_dir)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(&save_dir));

    if !canonical.starts_with(&save_dir_canonical) {
        return Err("Access denied: file is outside screenshot directory".to_string());
    }

    // Open and thumbnail the image
    let img = match image::open(&canonical) {
        Ok(img) => img,
        Err(_) => return Ok(None), // corrupted/unsupported — never crash
    };

    let thumb = img.thumbnail(320, 320);

    // Encode as JPEG quality 70
    let mut buf = std::io::Cursor::new(Vec::new());
    thumb
        .write_to(&mut buf, image::ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;

    use base64::Engine;
    Ok(Some(base64::engine::general_purpose::STANDARD.encode(buf.into_inner())))
}

#[tauri::command]
pub async fn adb_get_thumbnail(path: String) -> Result<Option<String>, String> {
    let path_buf = std::path::PathBuf::from(&path);
    // Run thumbnail generation on a blocking thread to avoid blocking the
    // async Tauri runtime.
    tokio::task::spawn_blocking(move || generate_thumbnail(&path_buf))
        .await
        .map_err(|e| format!("Thumbnail task failed: {}", e))?
}