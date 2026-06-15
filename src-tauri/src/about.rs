use serde::Serialize;
use tauri::{Emitter, State};

use crate::settings::{validate_adb_internal, validate_scrcpy_internal};
use crate::AppState;

#[derive(Serialize, Clone, Debug)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub commit: String,
    pub environment: BuildEnvironment,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum BuildEnvironment {
    Production,
    Development,
}

#[derive(Serialize, Clone, Debug)]
pub struct DependencyStatus {
    pub name: String,
    pub path: String,
    pub version: Option<String>,
    pub available: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct DebugInfo {
    pub app: AppInfo,
    pub dependencies: Vec<DependencyStatus>,
    pub platform: String,
    pub arch: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct UpdateCheckResult {
    pub available: bool,
    pub version: Option<String>,
    pub url: Option<String>,
    pub message: String,
}

#[tauri::command]
pub fn about_get_app_info() -> Result<AppInfo, String> {
    let environment = if cfg!(debug_assertions) {
        BuildEnvironment::Development
    } else {
        BuildEnvironment::Production
    };

    let commit = option_env!("GIT_COMMIT").unwrap_or("unknown").to_string();
    let short_commit = if commit.len() > 7 { &commit[..7] } else { &commit }.to_string();

    Ok(AppInfo {
        name: "ADB PowerHub".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit: short_commit,
        environment,
    })
}

fn extract_version_number(version_line: &str) -> Option<String> {
    // Accepts lines such as:
    //   "Android Debug Bridge version 1.0.41"
    //   "Version 1.0.41"
    //   "scrcpy 3.2"
    //   "3.2"
    let lower = version_line.to_lowercase();
    if let Some(idx) = lower.find("version ") {
        let rest = &version_line[idx + 8..];
        return rest.split_whitespace().next().map(|s| s.to_string());
    }
    version_line
        .split_whitespace()
        .find(|token| token.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
        .map(|s| s.to_string())
}

#[tauri::command]
pub fn about_get_dependency_status(state: State<AppState>) -> Result<Vec<DependencyStatus>, String> {
    let adb_path = {
        let path = crate::lock_state!(state.adb_path);
        path.clone()
    };

    let scrcpy_path = {
        let path = crate::lock_state!(state.scrcpy_path);
        path.clone().unwrap_or_default()
    };

    let mut dependencies = Vec::new();

    dependencies.push(match validate_adb_internal(&adb_path) {
        Ok(result) => DependencyStatus {
            name: "ADB".to_string(),
            path: result.path.clone(),
            version: extract_version_number(&result.version),
            available: true,
        },
        Err(_) => DependencyStatus {
            name: "ADB".to_string(),
            path: adb_path,
            version: None,
            available: false,
        },
    });

    dependencies.push(if scrcpy_path.is_empty() {
        DependencyStatus {
            name: "scrcpy".to_string(),
            path: String::new(),
            version: None,
            available: false,
        }
    } else {
        match validate_scrcpy_internal(&scrcpy_path) {
            Ok(result) => DependencyStatus {
                name: "scrcpy".to_string(),
                path: result.path.clone(),
                version: extract_version_number(&result.version),
                available: true,
            },
            Err(_) => DependencyStatus {
                name: "scrcpy".to_string(),
                path: scrcpy_path,
                version: None,
                available: false,
            },
        }
    });

    Ok(dependencies)
}

#[tauri::command]
pub fn about_get_debug_info(
    state: State<AppState>,
    _app_handle: tauri::AppHandle,
) -> Result<DebugInfo, String> {
    let app = about_get_app_info()?;
    let dependencies = about_get_dependency_status(state)?;

    Ok(DebugInfo {
        app,
        dependencies,
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    })
}

#[tauri::command]
pub async fn about_check_for_updates(
    app_handle: tauri::AppHandle,
) -> Result<UpdateCheckResult, String> {
    let result = UpdateCheckResult {
        available: false,
        version: None,
        url: None,
        message: "No updates available".to_string(),
    };

    let _ = app_handle.emit(
        "update-check-completed",
        serde_json::to_value(&result).unwrap_or_default(),
    );

    Ok(result)
}
