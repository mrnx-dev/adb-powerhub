// Icon extraction engine — host-side APK parsing via aapt2 + ZIP.
// Replaces the deprecated DEX-based approach (AppIcons.java).
//
// Architecture:
//   1. Pull APK from device to temp
//   2. Run aapt2 dump badging to resolve icon resource path
//   3. Extract icon from APK ZIP
//   4. If adaptive XML: parse foreground/background, composite to PNG
//   5. Cache with version-based invalidation + LRU eviction

// NOTE: Icon extraction functions below are used for full icon resolution.
// They are currently dead code but will be wired into adb_fetch_icons
// once adaptive icon support is enabled. Suppress warnings until then.
#![allow(dead_code)]

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use image::ImageEncoder;
use serde::{Deserialize, Serialize};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Parsed icon resource info from aapt2 output.
#[derive(Debug, Clone)]
pub struct IconResource {
    /// Density bucket (e.g., "xxxhdpi", "anydpi-v26")
    pub density: String,
    /// Full path in APK (e.g., "res/mipmap-xxxhdpi-v4/ic_launcher.png")
    pub apk_path: String,
}

/// Result of extracting an icon from an APK.
#[derive(Debug, Clone)]
pub struct ExtractedIcon {
    /// Package name
    pub package: String,
    /// Version code for cache invalidation
    pub version_code: i64,
    /// PNG bytes
    pub png_data: Vec<u8>,
    /// Whether composited from adaptive foreground+background
    pub is_adaptive: bool,
}

/// Cache metadata for a single entry.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheEntry {
    pub version_code: i64,
    pub file: String,
    pub size_bytes: u64,
    pub last_access: u64,
    pub is_adaptive: bool,
}

/// Full cache metadata file.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IconCache {
    pub version: u32,
    #[serde(default)]
    pub entries: HashMap<String, CacheEntry>,
    #[serde(default)]
    pub total_size_bytes: u64,
    #[serde(default = "default_max_size")]
    pub max_size_bytes: u64,
}

fn default_max_size() -> u64 {
    50 * 1024 * 1024 // 50 MB
}

impl IconCache {
    pub fn new() -> Self {
        IconCache {
            version: 1,
            entries: HashMap::new(),
            total_size_bytes: 0,
            max_size_bytes: default_max_size(),
        }
    }
}

// ─── Cache operations ──────────────────────────────────────────

/// Load icon cache from disk, or return a fresh empty one.
pub fn read_cache(cache_dir: &Path) -> IconCache {
    let cache_path = cache_dir.join("icon_cache.json");
    match std::fs::read_to_string(&cache_path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_else(|e| {
            eprintln!("[icons] Failed to parse cache, starting fresh: {}", e);
            IconCache::new()
        }),
        Err(_) => IconCache::new(),
    }
}

/// Atomically write cache to disk.
pub fn write_cache(cache_dir: &Path, cache: &IconCache) {
    let cache_path = cache_dir.join("icon_cache.json");
    let tmp_path = cache_dir.join("icon_cache.json.tmp");
    if let Some(parent) = cache_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        if std::fs::write(&tmp_path, &json).is_ok() {
            let _ = std::fs::rename(&tmp_path, &cache_path);
        }
    }
}

/// Check cache for a specific package+version. Returns PNG bytes if hit.
pub fn get_cached(cache_dir: &Path, pkg: &str, version_code: i64) -> Option<Vec<u8>> {
    let mut cache = read_cache(cache_dir);
    if let Some(entry) = cache.entries.get(pkg) {
        if entry.version_code == version_code {
            let png_path = cache_dir.join(&entry.file);
            if let Ok(data) = std::fs::read(&png_path) {
                // Update last_access
                if let Some(entry) = cache.entries.get_mut(pkg) {
                    entry.last_access = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                }
                write_cache(cache_dir, &cache);
                return Some(data);
            }
        }
    }
    None
}

/// Save extracted icon to cache.
pub fn save_to_cache(
    cache_dir: &Path,
    pkg: &str,
    version_code: i64,
    png_data: &[u8],
    is_adaptive: bool,
) {
    let _ = std::fs::create_dir_all(cache_dir);
    let file_name = format!("{}_{}.png", pkg.replace(':', "_"), version_code);
    let png_path = cache_dir.join(&file_name);
    if std::fs::write(&png_path, png_data).is_err() {
        eprintln!("[icons] Failed to write cache file: {:?}", png_path);
        return;
    }

    let mut cache = read_cache(cache_dir);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Remove old entry's size if it existed
    if let Some(old) = cache.entries.get(pkg) {
        cache.total_size_bytes = cache.total_size_bytes.saturating_sub(old.size_bytes);
        let old_path = cache_dir.join(&old.file);
        if old_path != png_path {
            let _ = std::fs::remove_file(&old_path);
        }
    }

    cache.entries.insert(
        pkg.to_string(),
        CacheEntry {
            version_code,
            file: file_name,
            size_bytes: png_data.len() as u64,
            last_access: now,
            is_adaptive,
        },
    );
    cache.total_size_bytes += png_data.len() as u64;
    write_cache(cache_dir, &cache);
}

/// Remove cache entries for packages no longer installed.
pub fn cleanup_stale_cache(cache_dir: &Path, current_packages: &[String]) {
    let mut cache = read_cache(cache_dir);
    let current_set: std::collections::HashSet<&str> =
        current_packages.iter().map(|s| s.as_str()).collect();

    let stale: Vec<String> = cache
        .entries
        .keys()
        .filter(|k| !current_set.contains(k.as_str()))
        .cloned()
        .collect();

    for pkg in &stale {
        if let Some(entry) = cache.entries.remove(pkg) {
            let png_path = cache_dir.join(&entry.file);
            let _ = std::fs::remove_file(&png_path);
            cache.total_size_bytes = cache.total_size_bytes.saturating_sub(entry.size_bytes);
        }
    }

    if !stale.is_empty() {
        eprintln!("[icons] Cleaned {} stale cache entries", stale.len());
        write_cache(cache_dir, &cache);
    }
}

/// Evict least-recently-accessed entries until total size is under target.
pub fn evict_lru(cache_dir: &Path, target_size: u64) {
    let mut cache = read_cache(cache_dir);
    if cache.total_size_bytes <= target_size {
        return;
    }

    // Sort entries by last_access (oldest first)
    let mut entries: Vec<(String, CacheEntry)> = cache.entries.clone().into_iter().collect();
    entries.sort_by_key(|(_, e)| e.last_access);

    let mut removed = 0u64;
    for (pkg, entry) in &entries {
        if cache.total_size_bytes <= target_size {
            break;
        }
        let png_path = cache_dir.join(&entry.file);
        let _ = std::fs::remove_file(&png_path);
        cache.total_size_bytes = cache.total_size_bytes.saturating_sub(entry.size_bytes);
        cache.entries.remove(pkg);
        removed += 1;
    }

    if removed > 0 {
        eprintln!(
            "[icons] LRU evicted {} entries, freed {} bytes",
            removed,
            cache.max_size_bytes.saturating_sub(cache.total_size_bytes)
        );
        write_cache(cache_dir, &cache);
    }
}

// ─── ADB helpers (minimal, self-contained) ─────────────────────

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn run_adb(adb_path: &str, serial: Option<&str>, args: &[&str], timeout_secs: u64) -> Result<String, String> {
    use std::sync::mpsc;

    let mut full_args: Vec<String> = Vec::new();
    if let Some(s) = serial {
        full_args.push("-s".to_string());
        full_args.push(s.to_string());
    }
    full_args.extend(args.iter().map(|s| s.to_string()));

    let (tx, rx) = mpsc::channel();
    let adb = adb_path.to_string();
    let timeout = Duration::from_secs(timeout_secs);

    std::thread::spawn(move || {
        #[cfg(windows)]
        let result = Command::new(&adb)
            .args(&full_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        #[cfg(not(windows))]
        let result = Command::new(&adb)
            .args(&full_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout) {
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
        Err(_) => Err(format!("ADB command timed out after {} seconds", timeout_secs)),
    }
}

// ─── APK Pull ──────────────────────────────────────────────────

/// Pull an APK file from device to a local temp file.
/// Uses `adb exec-out cat <path>` to stream the APK bytes.
pub fn pull_apk(adb_path: &str, serial: Option<&str>, apk_path: &str) -> Result<PathBuf, String> {
    let file_name = Path::new(apk_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let temp_dir = std::env::temp_dir();
    let dest = temp_dir.join(format!("adbph_{}", file_name));

    eprintln!(
        "[icons] Pulling APK: {} -> {}",
        apk_path,
        dest.display()
    );

    let mut cmd = if let Some(s) = serial {
        let mut c = Command::new(adb_path);
        c.args(["-s", s, "exec-out", "cat", apk_path]);
        c
    } else {
        let mut c = Command::new(adb_path);
        c.args(["exec-out", "cat", apk_path]);
        c
    };

    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to pull APK: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("adb exec-out failed with status {}", output.status)
        } else {
            stderr
        });
    }

    let mut file = std::fs::File::create(&dest)
        .map_err(|e| format!("Cannot create temp file {:?}: {}", dest, e))?;
    file.write_all(&output.stdout)
        .map_err(|e| format!("Cannot write temp file: {}", e))?;
    drop(file);

    Ok(dest)
}

// ─── aapt2 Resolution ──────────────────────────────────────────

/// Run `aapt2 dump badging` and parse the icon resource paths.
/// Returns all icon candidates (different densities).
pub fn resolve_icon_paths(aapt2_path: &str, apk_path: &Path) -> Result<Vec<IconResource>, String> {
    let output = Command::new(aapt2_path)
        .args(["dump", "badging", &apk_path.to_string_lossy()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Cannot run aapt2: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(if stderr.is_empty() {
            format!("aapt2 failed with status {}", output.status)
        } else {
            stderr
        });
    }

    let mut icons = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if !line.starts_with("application:") && !line.starts_with("launchable-activity:") {
            continue;
        }
        for part in line.split(' ') {
            if let Some(rest) = part.strip_prefix("icon='") {
                if let Some(icon_path) = rest.strip_suffix('\'') {
                    let density = extract_density(icon_path);
                    icons.push(IconResource {
                        density,
                        apk_path: icon_path.to_string(),
                    });
                }
            }
        }
    }

    if icons.is_empty() {
        return Err("No icon resource found in aapt2 output".to_string());
    }

    eprintln!(
        "[icons] aapt2 resolved {} icon candidates: {:?}",
        icons.len(),
        icons.iter().map(|i| &i.apk_path).collect::<Vec<_>>()
    );

    Ok(icons)
}

fn extract_density(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        let dir = parts[parts.len() - 2];
        if let Some(dash_pos) = dir.find('-') {
            return dir[dash_pos + 1..].to_string();
        }
        return dir.to_string();
    }
    "unknown".to_string()
}

fn select_best_icon(icons: &[IconResource]) -> Option<&IconResource> {
    let density_rank = |d: &str| -> u32 {
        match d {
            d if d.starts_with("xxxhdpi") => 6,
            d if d.starts_with("xxhdpi") => 5,
            d if d.starts_with("xhdpi") => 4,
            d if d.starts_with("hdpi") => 3,
            d if d.starts_with("mdpi") => 2,
            d if d.starts_with("ldpi") => 1,
            d if d.starts_with("anydpi") => 0,
            _ => 0,
        }
    };

    let best_png = icons
        .iter()
        .filter(|i| !i.density.starts_with("anydpi") && i.apk_path.ends_with(".png"))
        .max_by_key(|i| density_rank(&i.density));

    best_png.or_else(|| {
        icons
            .iter()
            .find(|i| i.density.starts_with("anydpi") && i.apk_path.ends_with(".xml"))
    })
}

// ─── ZIP Icon Extraction ───────────────────────────────────────

/// Extract a single file from the APK ZIP and return its bytes.
pub fn extract_zip_entry(apk_path: &Path, entry_name: &str) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(apk_path)
        .map_err(|e| format!("Cannot open APK for ZIP reading: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Cannot read APK as ZIP: {}", e))?;

    let mut entry = archive
        .by_name(entry_name)
        .map_err(|e| format!("Entry '{}' not found in APK: {}", entry_name, e))?;

    let mut buf = Vec::with_capacity(entry.size() as usize);
    std::io::copy(&mut entry, &mut buf)
        .map_err(|e| format!("Cannot read ZIP entry '{}': {}", entry_name, e))?;

    Ok(buf)
}

// ─── Adaptive Icon Handling ────────────────────────────────────

fn parse_adaptive_xml(
    apk_path: &Path,
    xml_entry: &str,
) -> Result<(String, String), String> {
    let xml_bytes = extract_zip_entry(apk_path, xml_entry)?;
    let xml_str =
        String::from_utf8(xml_bytes).map_err(|e| format!("Adaptive icon XML is not UTF-8: {}", e))?;

    let mut foreground = String::new();
    let mut background = String::new();

    for line in xml_str.lines() {
        let line = line.trim();
        for attr in &["android:foreground=", "android:drawable="] {
            if foreground.is_empty() && line.contains("foreground") {
                if let Some(start) = line.find(attr) {
                    let rest = &line[start + attr.len()..];
                    if let Some(val) = extract_drawable_ref(rest) {
                        foreground = val;
                    }
                }
            }
            if background.is_empty() && line.contains("background") {
                if let Some(start) = line.find(attr) {
                    let rest = &line[start + attr.len()..];
                    if let Some(val) = extract_drawable_ref(rest) {
                        background = val;
                    }
                }
            }
        }
    }

    if foreground.is_empty() {
        return Err("Could not find foreground drawable in adaptive icon XML".to_string());
    }

    if background.is_empty() {
        background = String::new();
    }

    let resolved_fg = resolve_drawable(apk_path, &foreground)?;
    let resolved_bg = if background.is_empty() {
        String::new()
    } else {
        resolve_drawable(apk_path, &background).unwrap_or_default()
    };

    Ok((resolved_fg, resolved_bg))
}

fn extract_drawable_ref(attr_value: &str) -> Option<String> {
    let val = attr_value.trim();
    let val = val.strip_prefix('\"').unwrap_or(val);
    let val = val.strip_suffix('\"').unwrap_or(val);
    if let Some(slash_pos) = val.rfind('/') {
        Some(val[slash_pos + 1..].to_string())
    } else {
        Some(val.to_string())
    }
}

fn resolve_drawable(apk_path: &Path, drawable_name: &str) -> Result<String, String> {
    let file =
        std::fs::File::open(apk_path).map_err(|e| format!("Cannot open APK: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Cannot read APK as ZIP: {}", e))?;

    let prefixes = ["mipmap", "drawable"];
    let densities = ["xxxhdpi", "xxhdpi", "xhdpi", "hdpi", "mdpi", ""];

    for prefix in &prefixes {
        for density in &densities {
            let dir = if density.is_empty() {
                format!("res/{}/", prefix)
            } else {
                format!("res/{}-{}-v4/", prefix, density)
            };
            let png_name = format!("{}{}.png", dir, drawable_name);
            if archive.by_name(&png_name).is_ok() {
                return Ok(png_name);
            }
            if !density.is_empty() {
                let dir_no_v4 = format!("res/{}-{}/", prefix, density);
                let png_name_no_v4 = format!("{}{}.png", dir_no_v4, drawable_name);
                if archive.by_name(&png_name_no_v4).is_ok() {
                    return Ok(png_name_no_v4);
                }
            }
        }
    }

    Err(format!(
        "Could not resolve drawable '{}' to a file in APK",
        drawable_name
    ))
}

fn compose_adaptive_icon(
    apk_path: &Path,
    xml_entry: &str,
    target_size: u32,
) -> Result<Vec<u8>, String> {
    let (fg_path, bg_path) = parse_adaptive_xml(apk_path, xml_entry)?;

    eprintln!(
        "[icons] Adaptive icon: fg={}, bg={}",
        fg_path,
        if bg_path.is_empty() { "(none)" } else { &bg_path }
    );

    let fg_bytes = extract_zip_entry(apk_path, &fg_path)?;
    let fg_img = image::load_from_memory(&fg_bytes)
        .map_err(|e| format!("Cannot decode foreground PNG: {}", e))?;

    let mut output = image::RgbaImage::new(target_size, target_size);

    if !bg_path.is_empty() {
        if let Ok(bg_bytes) = extract_zip_entry(apk_path, &bg_path) {
            if let Ok(bg_img) = image::load_from_memory(&bg_bytes) {
                let bg_resized =
                    image::imageops::resize(&bg_img.to_rgba8(), target_size, target_size, image::imageops::FilterType::Lanczos3);
                for (x, y, pixel) in bg_resized.enumerate_pixels() {
                    output.put_pixel(x, y, *pixel);
                }
            }
        }
    } else {
        for pixel in output.pixels_mut() {
            *pixel = image::Rgba([255, 255, 255, 255]);
        }
    }

    let fg_resized = image::imageops::resize(
        &fg_img.to_rgba8(),
        target_size,
        target_size,
        image::imageops::FilterType::Lanczos3,
    );
    for (x, y, pixel) in fg_resized.enumerate_pixels() {
        if pixel[3] > 0 {
            let bg = output.get_pixel(x, y);
            let alpha = pixel[3] as f32 / 255.0;
            let blended = image::Rgba([
                (pixel[0] as f32 * alpha + bg[0] as f32 * (1.0 - alpha)) as u8,
                (pixel[1] as f32 * alpha + bg[1] as f32 * (1.0 - alpha)) as u8,
                (pixel[2] as f32 * alpha + bg[2] as f32 * (1.0 - alpha)) as u8,
                255,
            ]);
            output.put_pixel(x, y, blended);
        }
    }

    let mut png_bytes = Vec::new();
    {
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder
            .write_image(output.as_raw(), target_size, target_size, image::ExtendedColorType::Rgba8.into())
            .map_err(|e| format!("Cannot encode PNG: {}", e))?;
    }

    Ok(png_bytes)
}

// ─── Main Extraction Orchestrator ──────────────────────────────

/// Extract an icon for a single package.
pub fn extract_icon(
    adb_path: &str,
    aapt2_path: &str,
    serial: Option<&str>,
    apk_path_on_device: &str,
    package: &str,
    version_code: i64,
) -> Result<ExtractedIcon, String> {
    let temp_apk = pull_apk(adb_path, serial, apk_path_on_device)?;

    let result = (|| -> Result<ExtractedIcon, String> {
        let icon_candidates = resolve_icon_paths(aapt2_path, &temp_apk)?;
        let best = select_best_icon(&icon_candidates)
            .ok_or("No suitable icon found in APK")?;

        eprintln!(
            "[icons] {} — selected: {} (density: {})",
            package, best.apk_path, best.density
        );

        let (png_data, is_adaptive) = if best.density.starts_with("anydpi")
            && best.apk_path.ends_with(".xml")
        {
            let composed = compose_adaptive_icon(&temp_apk, &best.apk_path, 192)?;
            (composed, true)
        } else {
            let bytes = extract_zip_entry(&temp_apk, &best.apk_path)?;
            (bytes, false)
        };

        Ok(ExtractedIcon {
            package: package.to_string(),
            version_code,
            png_data,
            is_adaptive,
        })
    })();

    let _ = std::fs::remove_file(&temp_apk);
    result
}

// ─── Tests ─────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    /// Resolve a fixture file under `src-tauri/tests/fixtures/` via
    /// `CARGO_MANIFEST_DIR` so it works locally and in CI. (Blueprint §2.2)
    fn fixture(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    // ── extract_drawable_ref (pure parser, sub-logic of parse_adaptive_xml) ──

    #[test]
    fn extract_drawable_ref_strips_resource_prefix_and_returns_last_segment() {
        assert_eq!(
            extract_drawable_ref("@drawable/foreground"),
            Some("foreground".to_string())
        );
        assert_eq!(
            extract_drawable_ref("@mipmap-v31/icon"),
            Some("icon".to_string())
        );
    }

    #[test]
    fn extract_drawable_ref_strips_surrounding_quotes() {
        assert_eq!(
            extract_drawable_ref("\"@drawable/fg\""),
            Some("fg".to_string())
        );
    }

    #[test]
    fn extract_drawable_ref_returns_value_as_is_when_no_slash() {
        assert_eq!(extract_drawable_ref("ic_launcher"), Some("ic_launcher".to_string()));
        // quoted no-slash
        assert_eq!(extract_drawable_ref("\"ic_launcher\""), Some("ic_launcher".to_string()));
    }

    // ── extract_density (pure parser, sub-logic of resolve_icon_paths) ──

    #[test]
    fn extract_density_extracts_density_qualifier_from_parent_dir() {
        assert_eq!(
            extract_density("res/mipmap-xxxhdpi/icon.png"),
            "xxxhdpi"
        );
        assert_eq!(extract_density("res/drawable-hdpi-v4/bg.png"), "hdpi-v4");
    }

    #[test]
    fn extract_density_returns_parent_dir_when_no_qualifier() {
        assert_eq!(extract_density("res/drawable/icon.png"), "drawable");
    }

    #[test]
    fn extract_density_returns_unknown_when_path_has_no_parent() {
        assert_eq!(extract_density("icon.png"), "unknown");
    }

    // ── extract_zip_entry (reads fixture ZIP; deterministic, no device) ──

    #[test]
    fn extract_zip_entry_reads_named_entry_from_fixture() {
        let bytes = extract_zip_entry(&fixture("dummy.zip"), "hello.txt")
            .expect("hello.txt should exist in dummy.zip");
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn extract_zip_entry_reads_nested_entry_from_fixture() {
        let bytes = extract_zip_entry(&fixture("dummy.zip"), "sub/nested.txt")
            .expect("sub/nested.txt should exist in dummy.zip");
        assert_eq!(bytes, b"world");
    }

    #[test]
    fn extract_zip_entry_errors_on_missing_entry() {
        let err = extract_zip_entry(&fixture("dummy.zip"), "does-not-exist.txt")
            .expect_err("missing entry should error");
        assert!(err.contains("not found"), "unexpected error: {err}");
    }

    #[test]
    fn extract_zip_entry_errors_on_malformed_zip() {
        let err = extract_zip_entry(&fixture("malformed.bin"), "anything")
            .expect_err("malformed archive should error");
        assert!(err.contains("Cannot read APK as ZIP"), "unexpected error: {err}");
    }

    // ── evict_lru / cleanup_stale_cache (filesystem-backed, tempdir) ──

    /// Build a temp cache dir with the given entries (each entry's png file is
    /// created as a zero-byte placeholder; only the metadata size matters for
    /// LRU accounting). The dir is unique per call via an atomic counter to
    /// avoid races between parallel tests.
    fn build_cache(entries: &[(&str, u64, u64)]) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "adb-powerhub-test-{}-{}",
            std::process::id(),
            id
        ));
        let _ = std::fs::create_dir_all(&dir);
        let mut cache = IconCache::new();
        for (pkg, size, last_access) in entries {
            let file = format!("{}.png", pkg);
            std::fs::write(dir.join(&file), b"placeholder").ok();
            cache.entries.insert(
                (*pkg).to_string(),
                CacheEntry {
                    version_code: 1,
                    file,
                    size_bytes: *size,
                    last_access: *last_access,
                    is_adaptive: false,
                },
            );
            cache.total_size_bytes += size;
        }
        write_cache(&dir, &cache);
        dir
    }

    #[test]
    fn evict_lru_removes_oldest_entries_until_under_target() {
        // 3 entries: A(oldest), B, C(newest). total=300. target=150 → evict A+B.
        let dir = build_cache(&[("pkgA", 100, 100), ("pkgB", 100, 200), ("pkgC", 100, 300)]);
        evict_lru(&dir, 150);
        let cache = read_cache(&dir);
        assert!(cache.entries.contains_key("pkgC"), "newest entry must remain");
        assert!(!cache.entries.contains_key("pkgA"), "oldest entry must be evicted");
        assert!(!cache.entries.contains_key("pkgB"), "second-oldest must be evicted");
        assert!(cache.total_size_bytes <= 150);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn evict_lru_noops_when_already_under_target() {
        let dir = build_cache(&[("pkgA", 50, 100)]);
        evict_lru(&dir, 1000); // target way above total
        let cache = read_cache(&dir);
        assert!(cache.entries.contains_key("pkgA"), "entry must remain when under target");
        assert_eq!(cache.total_size_bytes, 50);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_stale_cache_removes_packages_not_in_current_set() {
        let dir = build_cache(&[("pkgA", 100, 100), ("pkgB", 100, 200), ("pkgC", 100, 300)]);
        cleanup_stale_cache(&dir, &["pkgA".to_string(), "pkgC".to_string()]);
        let cache = read_cache(&dir);
        assert!(cache.entries.contains_key("pkgA"));
        assert!(cache.entries.contains_key("pkgC"));
        assert!(!cache.entries.contains_key("pkgB"), "stale pkgB must be removed");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_stale_cache_noops_when_no_stale_entries() {
        let dir = build_cache(&[("pkgA", 100, 100)]);
        cleanup_stale_cache(&dir, &["pkgA".to_string()]);
        let cache = read_cache(&dir);
        assert!(cache.entries.contains_key("pkgA"));
        assert_eq!(cache.total_size_bytes, 100);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
