// Icon extraction engine — host-side APK parsing via aapt2 + ZIP.
// Replaces the deprecated DEX-based approach (AppIcons.java).
//
// Architecture:
//   1. Pull APK from device to temp
//   2. Run aapt2 dump badging to resolve icon resource path
//   3. Extract icon from APK ZIP
//   4. If adaptive XML: parse foreground/background, composite to PNG
//   5. Cache with version-based invalidation + LRU eviction

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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
