use serde::Serialize;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use tauri::{Emitter, Manager, State};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

trait NoWindowSpawn {
    fn no_window(&mut self) -> &mut Self;
}

#[cfg(windows)]
impl NoWindowSpawn for Command {
    fn no_window(&mut self) -> &mut Self {
        self.creation_flags(CREATE_NO_WINDOW)
    }
}

#[cfg(not(windows))]
impl NoWindowSpawn for Command {
    fn no_window(&mut self) -> &mut Self {
        self
    }
}

use crate::AppState;

#[derive(Serialize, Clone)]
pub struct DownloadInfo {
    pub os: String,
    pub adb_download_url: String,
    pub adb_size_mb: String,
    pub scrcpy_link_url: String,
    pub scrcpy_link_label: String,
    pub scrcpy_install_hint: String,
}

#[derive(Serialize, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub version: String,
    pub path: String,
}

#[tauri::command]
pub fn settings_set_adb_path(path: String, state: State<AppState>) -> Result<(), String> {
    let mut adb_path = lock_state!(state.adb_path);
    *adb_path = path;
    Ok(())
}

#[tauri::command]
pub fn settings_set_scrcpy_path(path: String, state: State<AppState>) -> Result<(), String> {
    let mut scrcpy_path = lock_state!(state.scrcpy_path);
    *scrcpy_path = if path.is_empty() {
        None
    } else {
        Some(path)
    };
    Ok(())
}

#[tauri::command]
pub fn settings_validate_adb(path: String) -> Result<ValidationResult, String> {
    if path.trim().is_empty() {
        return Err("ADB path is empty".to_string());
    }
    let output = Command::new(&path)
        .no_window()
        .arg("version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let version = stdout
                .lines()
                .next()
                .unwrap_or("Unknown version")
                .to_string();
            Ok(ValidationResult {
                valid: true,
                version,
                path,
            })
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Err(stderr)
        }
        Err(e) => Err(format!("Cannot execute adb: {}", e)),
    }
}

#[tauri::command]
pub fn settings_validate_scrcpy(path: String) -> Result<ValidationResult, String> {
    if path.trim().is_empty() {
        return Err("scrcpy path is empty".to_string());
    }
    let output = Command::new(&path)
        .no_window()
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let version_line = if !stdout.is_empty() {
                stdout.lines().next().unwrap_or("Unknown")
            } else {
                stderr.lines().next().unwrap_or("Unknown")
            };
            if out.status.success() || version_line.contains("scrcpy") {
                Ok(ValidationResult {
                    valid: true,
                    version: version_line.to_string(),
                    path,
                })
            } else {
                Err(stderr)
            }
        }
        Err(e) => Err(format!("Cannot execute scrcpy: {}", e)),
    }
}

#[tauri::command]
pub fn settings_detect_adb() -> Result<Option<String>, String> {
    let adb_name = if cfg!(windows) { "adb.exe" } else { "adb" };
    match which::which(adb_name) {
        Ok(p) => Ok(Some(p.to_string_lossy().to_string())),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub fn settings_get_download_info() -> Result<DownloadInfo, String> {
    let os = std::env::consts::OS.to_string();

    let (adb_url, scrcpy_url, scrcpy_label, scrcpy_hint) = match os.as_str() {
        "windows" => (
            "https://dl.google.com/android/repository/platform-tools-latest-windows.zip".to_string(),
            "https://github.com/Genymobile/scrcpy/releases/latest".to_string(),
            "Download Scrcpy from GitHub".to_string(),
            String::new(),
        ),
        "macos" => (
            "https://dl.google.com/android/repository/platform-tools-latest-darwin.zip".to_string(),
            "https://formulae.brew.sh/formula/scrcpy".to_string(),
            "View on Homebrew".to_string(),
            "Install via Terminal: brew install scrcpy".to_string(),
        ),
        _ => (
            "https://dl.google.com/android/repository/platform-tools-latest-linux.zip".to_string(),
            "https://github.com/Genymobile/scrcpy/blob/master/doc/linux.md".to_string(),
            "View Install Guide".to_string(),
            "apt install scrcpy · pacman -S scrcpy · dnf install scrcpy".to_string(),
        ),
    };

    Ok(DownloadInfo {
        os,
        adb_download_url: adb_url,
        adb_size_mb: "15".to_string(),
        scrcpy_link_url: scrcpy_url,
        scrcpy_link_label: scrcpy_label,
        scrcpy_install_hint: scrcpy_hint,
    })
}

#[tauri::command]
pub fn settings_get_app_data_dir(app_handle: tauri::AppHandle) -> Result<String, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Cannot get app data dir: {}", e))?;
    Ok(data_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn settings_download_adb(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let cancel_flag = lock_state!(state.cancel_download).clone();
    cancel_flag.store(false, Ordering::SeqCst);

    let info = settings_get_download_info()?;

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Cannot get app data dir: {}", e))?;

    let bin_dir = app_data_dir.join("bin");
    fs::create_dir_all(&bin_dir).map_err(|e| format!("Cannot create bin dir: {}", e))?;

    let zip_path = bin_dir.join("adb-download.zip");
    let tmp_dir = bin_dir.join("adb-extract-tmp");

    if tmp_dir.exists() {
        let _ = fs::remove_dir_all(&tmp_dir);
    }
    fs::create_dir_all(&tmp_dir).map_err(|e| format!("Cannot create temp dir: {}", e))?;

    let mut response = reqwest::get(&info.adb_download_url)
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = fs::File::create(&zip_path).map_err(|e| format!("Cannot create zip file: {}", e))?;

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("Download stream error: {}", e))?
    {
        if cancel_flag.load(Ordering::SeqCst) {
            drop(file);
            let _ = fs::remove_file(&zip_path);
            let _ = fs::remove_dir_all(&tmp_dir);
            let _ = app_handle.emit("download-cancelled", serde_json::json!({ "type": "adb" }));
            return Err("Download cancelled".to_string());
        }

        file.write_all(&chunk).map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;

        let _ = app_handle.emit(
            "download-progress",
            serde_json::json!({
                "type": "adb",
                "read": downloaded,
                "total": total_size,
            }),
        );
    }

    drop(file);

    let zip_file = fs::File::open(&zip_path).map_err(|e| format!("Cannot open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| format!("Cannot read zip: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Cannot read zip entry {}: {}", i, e))?;

        if cancel_flag.load(Ordering::SeqCst) {
            let _ = fs::remove_file(&zip_path);
            let _ = fs::remove_dir_all(&tmp_dir);
            let _ = app_handle.emit("download-cancelled", serde_json::json!({ "type": "adb" }));
            return Err("Download cancelled".to_string());
        }

        let entry_path = entry.name().to_string();
        let out_path = tmp_dir.join(&entry_path);

        if entry.is_dir() {
            fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut out_file =
                fs::File::create(&out_path).map_err(|e| format!("Cannot extract {}: {}", entry_path, e))?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| format!("Cannot write {}: {}", entry_path, e))?;
        }
    }

    let adb_exe_name = if cfg!(windows) { "adb.exe" } else { "adb" };
    let adb_dest = bin_dir.join(adb_exe_name);

    let mut found = false;
    if let Ok(entries) = walkdir::WalkDir::new(&tmp_dir).into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in entries {
            if entry.file_name() == adb_exe_name {
                fs::copy(entry.path(), &adb_dest)
                    .map_err(|e| format!("Cannot copy adb binary: {}", e))?;
                found = true;
                break;
            }
        }
    }

    #[cfg(unix)]
    if found {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&adb_dest, perms).ok();
    }

    let _ = fs::remove_file(&zip_path);
    let _ = fs::remove_dir_all(&tmp_dir);

    if !found {
        return Err("ADB binary not found in downloaded archive".to_string());
    }

    let final_path = adb_dest.to_string_lossy().to_string();
    let mut adb_path = lock_state!(state.adb_path);
    *adb_path = final_path.clone();

    let _ = app_handle.emit(
        "download-complete",
        serde_json::json!({ "type": "adb", "path": final_path }),
    );

    Ok(final_path)
}

#[tauri::command]
pub fn settings_get_adb_version(path: String) -> Result<(u32, u32, u32), String> {
    let output = Command::new(&path)
        .no_window()
        .arg("version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Cannot execute adb: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(ver_str) = trimmed.strip_prefix("Version ") {
            let parts: Vec<&str> = ver_str.split('.').collect();
            if parts.len() >= 2 {
                let major = parts[0].parse().unwrap_or(0);
                let minor = parts[1].parse().unwrap_or(0);
                let patch = if parts.len() > 2 { parts[2].parse().unwrap_or(0) } else { 0 };
                return Ok((major, minor, patch));
            }
        }
        let lower = trimmed.to_lowercase();
        if lower.starts_with("android debug bridge version ") {
            let rest = &lower["android debug bridge version ".len()..];
            let parts: Vec<&str> = rest.split('.').collect();
            if parts.len() >= 2 {
                let major = parts[0].parse().unwrap_or(0);
                let minor = parts[1].parse().unwrap_or(0);
                let patch = if parts.len() > 2 { parts[2].parse().unwrap_or(0) } else { 0 };
                return Ok((major, minor, patch));
            }
        }
    }
    Err("Could not parse ADB version".to_string())
}

#[tauri::command]
pub fn settings_cancel_download(state: State<AppState>) -> Result<(), String> {
    let cancel_flag = lock_state!(state.cancel_download).clone();
    cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}