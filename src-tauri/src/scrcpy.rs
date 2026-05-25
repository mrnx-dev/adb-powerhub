use serde::Serialize;
use std::process::Command;
use tauri::{Emitter, State};

use crate::AppState;

#[derive(Serialize, Clone)]
pub struct ScrcpyStatus {
    pub available: bool,
    pub path: Option<String>,
}

#[tauri::command]
pub fn adb_find_scrcpy(state: State<AppState>) -> Result<ScrcpyStatus, String> {
    let stored = lock_state!(state.scrcpy_path).clone();

    if let Some(ref path) = stored {
        if std::path::Path::new(path).exists() {
            return Ok(ScrcpyStatus {
                available: true,
                path: Some(path.clone()),
            });
        }
    }

    let found = which::which("scrcpy")
        .ok()
        .or_else(|| which::which("scrcpy.exe").ok())
        .map(|p| p.to_string_lossy().to_string());

    if let Some(ref path) = found {
        let mut stored_path = lock_state!(state.scrcpy_path);
        *stored_path = Some(path.clone());
    }

    Ok(ScrcpyStatus {
        available: found.is_some(),
        path: found,
    })
}

#[tauri::command]
pub fn adb_launch_scrcpy(
    state: State<AppState>,
    app_handle: tauri::AppHandle,
    record: bool,
    save_dir: Option<String>,
    video_bitrate: Option<String>,
    max_size: Option<String>,
    recording_format: Option<String>,
) -> Result<String, String> {
    let scrcpy_status = adb_find_scrcpy(state.clone())?;

    if !scrcpy_status.available {
        return Err("scrcpy not found. Please install scrcpy or set its path in settings.".to_string());
    }

    let scrcpy_path = scrcpy_status.path.ok_or("scrcpy path is missing despite being available".to_string())?;

    let mut cmd = Command::new(&scrcpy_path);
    cmd.arg("--stay-awake");

    let device_serial = state.connected_device.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(serial) = device_serial {
        cmd.arg("--serial").arg(&serial);
    }

    if let Some(bitrate) = video_bitrate {
        cmd.arg("--video-bit-rate").arg(&bitrate);
    }
    if let Some(size) = &max_size {
        cmd.arg("--max-size").arg(size);
    }

    let format = recording_format.as_deref().unwrap_or("mkv");

    if record {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        let record_dir = save_dir.unwrap_or_else(|| {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home)
                .join("Videos")
                .join("adb-powerhub")
                .to_string_lossy()
                .to_string()
        });
        let _ = std::fs::create_dir_all(&record_dir);
        let ext = format!(".{}", format);
        let record_file = std::path::PathBuf::from(&record_dir)
            .join(format!("recording_{}{}", timestamp, ext))
            .to_string_lossy()
            .to_string();

        cmd.arg("--record-format").arg(format);
        cmd.arg("--record").arg(&record_file);
    }

    let mut child = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch scrcpy: {}", e))?;

    let pid = child.id();
    let emit_handle = app_handle.clone();

    // Spawn a monitor thread that waits for scrcpy to exit and emits an event
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = emit_handle.emit("scrcpy-exited", ());
    });

    let mut proc = lock_state!(state.scrcpy_process);
    *proc = Some(pid);

    Ok("scrcpy launched".to_string())
}

#[tauri::command]
pub async fn adb_stop_scrcpy(state: State<'_, AppState>) -> Result<String, String> {
    let pid = {
        let mut proc = lock_state!(state.scrcpy_process);
        proc.take()
    };

    if let Some(pid) = pid {
        // Step 1: Soft kill — allow graceful shutdown for recording finalization
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string()])
                .output();
        }
        #[cfg(unix)]
        {
            unsafe {
                libc::kill(pid as i32, 15); // SIGTERM
            }
        }

        // Step 2: Wait up to 4 seconds for graceful shutdown
        for _ in 0..8 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            if !is_process_alive(pid) {
                return Ok("scrcpy stopped".to_string());
            }
        }

        // Step 3: Force kill
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output();
        }
        #[cfg(unix)]
        {
            unsafe {
                libc::kill(pid as i32, 9); // SIGKILL
            }
        }

        // Step 4: Wait briefly for force kill to take effect
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        if !is_process_alive(pid) {
            Ok("scrcpy stopped (force)".to_string())
        } else {
            Err("Failed to stop scrcpy process".to_string())
        }
    } else {
        Ok("No scrcpy process running".to_string())
    }
}

fn is_process_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.contains(&pid.to_string())
            }
            Err(_) => false,
        }
    }
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
}