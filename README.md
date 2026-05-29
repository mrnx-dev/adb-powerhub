# ADB PowerHub

A cross-platform desktop application for controlling Android devices via ADB (Android Debug Bridge). Built with Tauri v2, Vue 3, and Rust.

![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![Tauri](https://img.shields.io/badge/Tauri-v2-24c8d8)
![Vue](https://img.shields.io/badge/Vue-3-42b883)
![Rust](https://img.shields.io/badge/Rust-2021-dea584)

## Features

### Device Connection

- Connect via IP address (wireless ADB)
- USB auto-connect with device detection
- Auto-connect on launch (configurable)
- Device switching and disconnection

### Device Monitoring

- Real-time battery level, status, and health
- CPU usage monitoring
- Device model, Android version, and SDK info
- Configurable polling interval (1–30 seconds)
- Auto-disconnect after 3 consecutive poll failures

### Connectivity Toggles

- Wi-Fi enable/disable
- Mobile data enable/disable
- Airplane mode toggle
- Bluetooth enable/disable

### Developer Tools

- Show touches toggle
- Layout bounds toggle
- Stay awake (keep screen on while charging)

### Remote Controls

- Navigation: Home, Back, Recent
- Volume: Up, Down, Mute
- Media: Previous, Play/Pause, Next
- Power button
- Text input to device
- Brightness slider (0–255)
- Screen rotation toggle

### Screen Mirroring (scrcpy)

- Launch scrcpy with configurable quality presets (Low/Medium/High/Custom)
- Screen recording (MKV or MP4 format)
- Stay awake during mirroring
- In-app scrcpy path configuration

### Terminal

- Interactive ADB command execution
- Timestamped log viewer with copy/export
- Real-time command output

### System Actions

- Screenshot capture and auto-open folder
- Reboot (normal, recovery, bootloader)

### Settings

- ADB binary path (auto-detect, manual, or in-app download from Google)
- Scrcpy binary path (with install guidance)
- Stay on top (always-on-top window)
- Auto-connect and auto-detect on launch
- Video quality presets and custom bitrate/resolution
- Recording format (MKV/MP4)
- Screenshot and recording save directories
- Persistent settings via Tauri store

### UI/UX

- Dark glass-morphism theme with emerald accent
- Collapsible sidebars (left: navigation, right: controls)
- Toast notifications for key actions
- Keyboard shortcuts (Ctrl+K: focus terminal, Escape: close menus)
- Click-outside to close dropdown menus

## Tech Stack

| Layer            | Technology                                 |
| ---------------- | ------------------------------------------ |
| Frontend         | Vue 3 (Composition API + `<script setup>`) |
| State Management | Pinia                                      |
| Styling          | Tailwind CSS v4                            |
| Icons            | Lucide Vue Next, Material Design Icons     |
| Build            | Vite                                       |
| Desktop          | Tauri v2                                   |
| Backend          | Rust (2021 edition)                        |
| Async            | Tokio                                      |
| HTTP             | Reqwest                                    |
| CI/CD            | GitHub Actions (build 4 platforms)         |

## Project Structure

```
adb-powerhub/
  src/                          # Vue 3 frontend
    main.ts                     # App bootstrap
    App.vue                     # Root component
    style.css                   # Tailwind + custom styles
    stores/
      device.ts                 # Device state, ADB commands, polling
      settings.ts               # Settings persistence, binary config
      navigation.ts             # Page router, terminal focus
      toast.ts                  # Toast notification state
    views/
      DashboardView.vue         # Main dashboard layout
      SettingsView.vue          # Settings page
    components/
      TitleBar.vue              # Custom window title bar
      AppSidebarLeft.vue        # Collapsible navigation sidebar
      AppSidebarRight.vue       # Collapsible controls sidebar
      ConnectCard.vue           # Device connection form
      DeviceStatsCard.vue       # Battery, CPU, device info bar
      MirrorCard.vue            # Scrcpy mirror controls
      TerminalCard.vue           # ADB command terminal
      AppToast.vue              # Toast notification overlay
      settings/
        BinaryPathsCard.vue     # ADB/scrcpy path config
        GeneralPrefsCard.vue    # General preferences
        DisplayRecordingCard.vue # Video quality & recording settings
    composables/
      useKeyboardShortcuts.ts  # Global keyboard shortcuts
  src-tauri/                    # Rust backend
    src/
      main.rs                   # Entry point
      lib.rs                    # AppState, command registration
      adb.rs                    # ADB command implementations
      scrcpy.rs                 # Scrcpy launch/stop
      settings.rs               # Binary path, download, validation
    tauri.conf.json             # Tauri window & bundle config
    capabilities/
      default.json             # Permission declarations
```

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 22+
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Tauri CLI](https://v2.tauri.app/start/create-project/) v2
- ADB binary (auto-detected or downloadable in-app)
- scrcpy (optional, for mirroring)

### Install

```bash
cd adb-powerhub
npm install
```

### Development

```bash
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

Produced installers will be in `src-tauri/target/release/bundle/`.

## Keyboard Shortcuts

| Shortcut | Action                        |
| -------- | ----------------------------- |
| `Ctrl+K` | Focus terminal input          |
| `Escape` | Close reboot menu / dropdowns |

## License

This project is proprietary software. All rights reserved.
