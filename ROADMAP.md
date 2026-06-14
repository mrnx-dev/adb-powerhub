# ADB PowerHub — Roadmap & Actual Progress

> Rekonstruksi dari `git log`, source code, dan `.miko/`.
> Terakhir diperbarui: 2026-06-14.

## Legenda

| Simbol | Arti |
|--------|------|
| ✅ | Sudah implementasi & merge ke `main` |
| 🚧 | Implementasi parsial / butuh polish / belum stabil |
| ⏳ | Direncanakan, belum dikerjakan |
| ❌ | Sempat dikerjakan lalu dihapus |

## Status Ringkasan

- **Milestone terakhir selesai:** Connect Panel Redesign + Stage 4 Emil polish — `2026-06-11`
- **Total fitur yang sudah jalan:** ~15 area fitur utama
- **Fitur yang baru direncanakan:** Network diagnostics, Port Forwarding, Process Manager, Performance Charts, Backup & Restore, Scheduled Recording, Multiple Device support

## ✅ Sudah Selesai

| Area | Fitur | Selesai / Catatan |
|------|-------|-------------------|
| **Core & Koneksi** | Connect via IP + port, USB auto-detect, auto-connect on launch, device switching, disconnect, cancel connect, pair (`adb pair`) | 2026-05-28 → 2026-06-11 |
| | Connection history & saved devices | 2026-06-11 (QuickReconnectList, SavedDeviceItem) |
| | Auto-reconnect saat device kembali online | 2026-06-04 |
| | Connect Panel redesign — unified vertical stack, state machine, a11y, reduced-motion, Emil polish | 2026-06-11 |
| **Monitoring** | Real-time battery, CPU, RAM, storage, display resolution, voltage, model, Android/SDK | 2026-06-03 → 2026-06-04 |
| | Device stats polling interval (1–30s) & auto-disconnect setelah 3x gagal | 2026-06-03 |
| **Toggles & Kontrol** | Wi-Fi, Mobile Data, Airplane, Bluetooth toggles | 2026-05-28 |
| | Show touches, Layout bounds, Stay awake, Brightness, DPI/density changer, Screen rotation | 2026-06-03 → 2026-06-04 |
| | Remote keys: Home, Back, Recent, Volume, Media, Power, text input | 2026-05-25 |
| **Mirror / scrcpy** | Launch/stop scrcpy, quality presets, toggles (show-touches, screen-off, always-on-top, no-control), screen recording MKV/MP4 | 2026-05-28 → 2026-06-04 |
| **App Manager** | List installed apps (all/system/user/disabled), sort & filter, search, real labels, icons | 2026-06-04 → 2026-06-07 |
| | 3-panel layout, hover preview, detail view, keyboard navigation | 2026-06-06 |
| | Actions: open, uninstall/clear, force stop, enable/disable | 2026-06-04 |
| | APK install via file picker + drag-and-drop desktop → app | 2026-06-04 |
| | Icon extraction engine (on-device ZIP DEX + aapt2 fallback, cache, progressive delivery) | 2026-06-05 → 2026-06-06 |
| **Screenshot Gallery** | Grid view, lazy thumbnails, sort (newest/oldest/largest), filter (all/today/week) | 2026-06-08 |
| | Lightbox viewer with zoom, FLIP transition, keyboard nav, metadata, delete/open folder | 2026-06-08 → 2026-06-10 |
| | Truncation banner (max 500 files), manual refresh, capture integration | 2026-06-09 |
| **Logcat** | Real-time stream, level/tag filter, multi-tag include/exclude, active-app filter, pause/resume, export | 2026-05-31 → 2026-06-01 |
| **Terminal** | Interactive ADB command execution + timestamped log | 2026-05-25 |
| **Command Presets** | CRUD presets, quick-run, Alt+1–9 shortcuts, persisted | 2026-05-29 |
| **Settings** | ADB/scrcpy path (auto-detect, manual, in-app download), aapt2 settings + download | 2026-05-28 → 2026-06-05 |
| | Multi-theme system (18 themes), auto/light/dark mode, accent-aware tokens | 2026-05-25 → 2026-06-04 |
| | Stay on top, auto-detect, auto-reconnect, polling interval, screenshot/recording dirs, video quality | 2026-05-28 → 2026-06-04 |
| **UI/UX** | Page transitions, collapsible sidebars, glass-morphism, toast notifications, drag-drop overlay, custom title bar | 2026-05-25 → 2026-06-11 |

## ❌ Dihapus

| Fitur | Alasan |
|-------|--------|
| **Clipboard Sync** | Ditambahkan 2026-06-03, dihapus 2026-06-04 (`21d6fbe`). Tidak reliable di Android 14+ (SDK 34) karena pembatasan clipboard access. |

## 🚧 Parsial / Butuh Perhatian

| Area | Status | Catatan |
|------|--------|---------|
| **App Manager detail** | 🚧 | Data/cache usage & install-date belum ditampilkan di UI (backend sudah parse `firstInstallTime`, `lastUpdateTime`, `apk_size`). |
| **Theme system** | 🚧 | 18 themes & auto mode sudah jalan, tapi beberapa preview/theme-blocks mungkin butuh penyesuaian token. |

## ⏳ Backlog / Next Up

Berdasarkan `EXECUTION_PLAN.md` dan kode saat ini:

| Phase | Fitur | Prioritas | Dependensi |
|-------|-------|-----------|--------------|
| **Phase 4 (UX + Network)** | Network Info (SSID, MAC, proxy, signal) | P2 | `adb_poll_device_stats` + dumpsys wifi |
| | Proxy Setting (global/http) | P2 | Network Info |
| **Phase 5 (Pro Tools)** | Port Forwarding / Reverse rules table | P2 | State `AppState` saat ini single-device |
| | Process Manager (`top`/`ps` polling, sort/filter table) | P2 | Logcat streaming pattern + App Manager table |
| | Performance Charts (rolling window time-series) | P2 | Device stats polling |
| **Phase 6 (Automation)** | Backup & Restore wizard (`adb backup`/`restore`) | P2 | Long-running cancellable command pattern |
| | Scheduled Recording (timer + scrcpy integration) | P2 | scrcpy launch + scheduling |
| **Phase 7 (Architecture)** | Multiple Device Support (tab/switcher, refactor `AppState`, `device.ts`, routing) | P2 | **Harus terakhir** — semua fitur single-device stabil dulu. |

## Timeline Singkat

```
2026-05-25  Initial app, terminal, remote controls, light/dark theme
2026-05-28  Settings, mirror/scrcpy, connectivity toggles
2026-05-29  Command Presets
2026-05-31  Logcat Viewer
2026-06-01  Logcat filters, active-app filter
2026-06-03  DPI Changer, Enhanced Device Info, Clipboard Sync
2026-06-04  Auto-Reconnect, App Manager, APK Drag & Drop, Theme Toggle
            (Clipboard Sync dihapus)
2026-06-05  Icon extraction engine + aapt2 support
2026-06-06  App Manager 3-panel + keyboard nav
2026-06-07  Open app, timeline, polish
2026-06-08  Screenshot Gallery
2026-06-09  Thumbnail cache, truncation, refresh
2026-06-10  Lightbox Emil polish
2026-06-11  Connect Panel Redesign + Stage 4 polish
```

## Cara Membaca

- `EXECUTION_PLAN.md` adalah rencana kurasi berdasarkan dependency & risk.
- `ROADMAP.md` ini adalah **keadaan nyata** di `main` setelah saya rekontruksi dari source & git.
