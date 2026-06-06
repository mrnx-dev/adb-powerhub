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

// ─── aapt2 Commands (stubs) ──────────────────────────────────

#[tauri::command]
pub fn settings_validate_aapt2(path: String) -> Result<ValidationResult, String> {
    if path.trim().is_empty() {
        return Err("aapt2 path is empty".to_string());
    }
    let output = Command::new(&path)
        .no_window()
        .arg("version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let version = stdout.lines().next().unwrap_or("Unknown version").to_string();
            if out.status.success() {
                Ok(ValidationResult { valid: true, version, path })
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                Err(if stderr.is_empty() { format!("aapt2 exited with status {}", out.status) } else { stderr })
            }
        }
        Err(e) => Err(format!("Cannot execute aapt2: {}", e)),
    }
}

#[tauri::command]
pub fn settings_detect_aapt2() -> Result<Option<String>, String> {
    let name = if cfg!(windows) { "aapt2.exe" } else { "aapt2" };
    // Check PATH first
    if let Ok(p) = which::which(name) {
        return Ok(Some(p.to_string_lossy().to_string()));
    }
    // Check ANDROID_HOME / build-tools
    if let Ok(android_home) = std::env::var("ANDROID_HOME") {
        let bt_dir = std::path::PathBuf::from(&android_home).join("build-tools");
        if let Ok(entries) = std::fs::read_dir(&bt_dir) {
            let mut versions: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            // Pick latest version (simple string sort works for semver-like versions)
            versions.sort();
            for ver in versions.iter().rev() {
                let candidate = bt_dir.join(ver).join(name);
                if candidate.exists() {
                    return Ok(Some(candidate.to_string_lossy().to_string()));
                }
            }
        }
    }
    Ok(None)
}

#[derive(Serialize, Clone)]
pub struct Aapt2DownloadInfo {
    pub os: String,
    pub download_url: String,
    pub size_mb: String,
}

#[tauri::command]
pub fn settings_get_aapt2_download_info() -> Result<Aapt2DownloadInfo, String> {
    let os = std::env::consts::OS.to_string();
    // Google Maven publishes per-platform aapt2 JARs (~5 MB each).
    // The JAR contains the native aapt2 binary at its root.
    // Platform classifiers: windows, linux, osx (not macos!)
    // See: https://developer.android.com/tools/aapt2
    let aapt2_version = "7.4.2-8841542";
    let platform = match os.as_str() {
        "windows" => "windows",
        "macos" => "osx",
        _ => "linux",
    };
    let url = format!(
        "https://dl.google.com/dl/android/maven2/com/android/tools/build/aapt2/{0}/aapt2-{0}-{1}.jar",
        aapt2_version, platform
    );
    Ok(Aapt2DownloadInfo { os, download_url: url, size_mb: "5".to_string() })
}

#[tauri::command]
pub async fn settings_download_aapt2(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let cancel_flag = lock_state!(state.cancel_download).clone();
    cancel_flag.store(false, Ordering::SeqCst);

    let info = settings_get_aapt2_download_info()?;

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Cannot get app data dir: {}", e))?;

    let bin_dir = app_data_dir.join("bin");
    fs::create_dir_all(&bin_dir).map_err(|e| format!("Cannot create bin dir: {}", e))?;

    let zip_path = bin_dir.join("aapt2-download.jar");
    let tmp_dir = bin_dir.join("aapt2-extract-tmp");

    if tmp_dir.exists() {
        let _ = fs::remove_dir_all(&tmp_dir);
    }
    fs::create_dir_all(&tmp_dir).map_err(|e| format!("Cannot create temp dir: {}", e))?;

    let mut response = reqwest::get(&info.download_url)
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
            let _ = app_handle.emit("download-cancelled", serde_json::json!({ "type": "aapt2" }));
            return Err("Download cancelled".to_string());
        }
        file.write_all(&chunk).map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;
        let _ = app_handle.emit(
            "download-progress",
            serde_json::json!({
                "type": "aapt2",
                "read": downloaded,
                "total": total_size,
            }),
        );
    }
    drop(file);

    // Extract aapt2 native binary from the Maven JAR (JAR = ZIP).
    // The native binary is at the root of the JAR: "aapt2.exe" (Windows) or "aapt2" (others).
    let zip_file = fs::File::open(&zip_path).map_err(|e| format!("Cannot open jar: {}", e))?;
    let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| format!("Cannot read jar as ZIP: {}", e))?;

    let aapt2_name = if cfg!(windows) { "aapt2.exe" } else { "aapt2" };

    'extract: for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("Cannot read jar entry {}: {}", i, e))?;
        let entry_path = entry.name().to_string();

        if cancel_flag.load(Ordering::SeqCst) {
            let _ = fs::remove_file(&zip_path);
            let _ = fs::remove_dir_all(&tmp_dir);
            let _ = app_handle.emit("download-cancelled", serde_json::json!({ "type": "aapt2" }));
            return Err("Download cancelled".to_string());
        }

        // The native binary is at the root of the JAR (not in a subdirectory)
        if entry_path == aapt2_name {
            let out_path = tmp_dir.join(aapt2_name);
            let mut out_file = fs::File::create(&out_path)
                .map_err(|e| format!("Cannot extract aapt2: {}", e))?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| format!("Cannot write aapt2: {}", e))?;
            break 'extract;
        }
    }

    if !tmp_dir.join(aapt2_name).exists() {
        // Fallback: search for aapt2 binary anywhere in the JAR
        // Need to close the first archive before opening a new one
        drop(archive);
        let jar_file = fs::File::open(&zip_path).map_err(|e| format!("Cannot reopen jar: {}", e))?;
        let mut archive2 = zip::ZipArchive::new(jar_file)
            .map_err(|e| format!("Cannot re-read jar: {}", e))?;
        for i in 0..archive2.len() {
            let mut entry = archive2.by_index(i)
                .map_err(|e| format!("Cannot read jar entry {}: {}", i, e))?;
            let ep = entry.name().to_string();
            if ep.ends_with(&format!("/{}", aapt2_name)) || ep == aapt2_name {
                let mut out_file = fs::File::create(tmp_dir.join(aapt2_name))
                    .map_err(|e| format!("Cannot extract aapt2: {}", e))?;
                std::io::copy(&mut entry, &mut out_file)
                    .map_err(|e| format!("Cannot write aapt2: {}", e))?;
                break;
            }
        }
    }

    let aapt2_src = tmp_dir.join(aapt2_name);
    if !aapt2_src.exists() {
        let _ = fs::remove_file(&zip_path);
        let _ = fs::remove_dir_all(&tmp_dir);
        return Err("aapt2 binary not found in downloaded archive".to_string());
    }

    let aapt2_dest = bin_dir.join(aapt2_name);
    fs::copy(&aapt2_src, &aapt2_dest).map_err(|e| format!("Cannot copy aapt2: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&aapt2_dest, perms).ok();
    }

    let _ = fs::remove_file(&zip_path);
    let _ = fs::remove_dir_all(&tmp_dir);

    let final_path = aapt2_dest.to_string_lossy().to_string();
    let mut aapt2_path = lock_state!(state.aapt2_path);
    *aapt2_path = Some(final_path.clone());

    let _ = app_handle.emit(
        "download-complete",
        serde_json::json!({ "type": "aapt2", "path": final_path }),
    );

    Ok(final_path)
}

#[tauri::command]
pub fn settings_set_aapt2_path(path: String, state: State<AppState>) -> Result<(), String> {
    let mut aapt2_path = lock_state!(state.aapt2_path);
    *aapt2_path = if path.is_empty() { None } else { Some(path) };
    Ok(())
}