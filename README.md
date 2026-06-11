# ADB PowerHub

A cross-platform desktop application for controlling Android devices via ADB (Android Debug Bridge). Built with Tauri v2, Vue 3, and Rust.

![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![Tauri](https://img.shields.io/badge/Tauri-v2-24c8d8)
![Vue](https://img.shields.io/badge/Vue-3-42b883)
![Rust](https://img.shields.io/badge/Rust-2021-dea584)

## Features

### Device Connection

- Connect via IP address (wireless ADB) with port config
- USB auto-connect with device detection
- Auto-connect on launch (configurable)
- Device switching and disconnection

### Device Monitoring

- Real-time battery level, status, health, and temperature
- CPU usage monitoring
- RAM (total/available) and storage (total/used)
- Display resolution and battery voltage
- Device model, Android version, and SDK info
- Configurable polling interval (1–30 seconds)
- Auto-disconnect after 3 consecutive poll failures
- Auto-reconnect when device comes back online
- Connection history with saved devices

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
- DPI/density changer with slider and reset
- Screen rotation toggle

### Screen Mirroring (scrcpy)

- Launch scrcpy with configurable quality presets (Low/Medium/High/Custom)
- Show touches, stay awake, turn screen off, always-on-top, no-control toggles
- Screen recording (MKV or MP4 format)
- In-app scrcpy path configuration

### App Manager

- Grid view of installed apps with icons and labels
- Sort by name, size, and install date; filter by system/user/all
- 3-panel layout: app list, detail view, action sidebar
- App detail: version, install date, APK size, data/cache usage
- Keyboard navigation (arrow keys, Enter, Space, /, Escape)
- Actions: open, uninstall, clear data, force stop, enable, disable
- Drag-and-drop APK install (.apk files from desktop)
- Real-time icon extraction (resources.arsc parser, on-device ZIP fallback)

### Screenshot Gallery

- Gallery grid of all screenshots in configured save directory
- Lazy-loaded thumbnails with intersection observer
- Sort by newest, oldest, largest; filter by all, today, this week
- Full-screen lightbox viewer with keyboard navigation
- Lightbox metadata: filename, dimensions, file size, date
- Quick actions: open in OS viewer, open folder, copy path, delete
- Take Screenshot button integrated with ADB capture
- Auto-refresh after capture (debounced)
- Max 500 files per listing with truncation indicator

### Terminal

- Interactive ADB command execution
- Timestamped log viewer with copy/export
- Real-time command output

### Logcat

- Real-time logcat stream with level/tag/process coloring
- Filter by level (Verbose/Debug/Info/Warning/Error/Fatal)
- Multi-tag filter with include/exclude chips
- Active app filter (foreground package detection)
- Pause/resume stream
- Export to text file
- Search/filter

### Command Presets

- Built-in presets for common ADB commands
- Custom preset creation with name + command
- Quick-run via Alt+1-9 keyboard shortcuts
- Edit and delete presets
- Persisted to presets.json via Tauri store

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
- DPI/density with slider
- Persistent settings via Tauri store

### UI/UX

- Multi-theme system with 18 themes (6 dark, 6 monochrome, 6 light) + auto mode
- Theme-aware accent colors (emerald default)
- Dark glass-morphism design language
- Collapsible sidebars (left: navigation, right: controls)
- Page transitions with out-in mode
- Custom thin scrollbar styling
- Toast notifications for key actions (success/error/info/progress)
- Click-outside to close dropdown menus
- APK drag-and-drop overlay with install queue
- Custom title bar with traffic-light controls

## Tech Stack

| Layer            | Technology                                 |
| ---------------- | ------------------------------------------ |
| Frontend         | Vue 3 (Composition API + `<script setup>`) |
| State Management | Pinia                                      |
| Styling          | Tailwind CSS v4                            |
| Icons            | Lucide Vue, Material Design Icons          |
| Build            | Vite                                       |
| Desktop          | Tauri v2                                   |
| Backend          | Rust (2021 edition)                        |
| Async            | Tokio                                      |
| HTTP             | Reqwest                                    |
| CI/CD            | GitHub Actions (build 4 platforms)         |

## Project Structure

```
adb-powerhub/
  src/                              # Vue 3 frontend (~9400 LOC)
    main.ts                         # App bootstrap (Vue + Pinia)
    App.vue                         # Root - view router + APK drop zone + page transitions
    style.css                       # Tailwind v4 imports + custom theme tokens
    stores/
      device.ts                     # ADB connection, commands, stats, polling (~1280 LOC)
      settings.ts                   # Binary paths, preferences, persistence
      logcat.ts                     # Logcat stream, filters, active app detection
      apps.ts                       # App list, detail, icons, install/uninstall
      screenshots.ts                # Screenshot gallery listing, sort, filter, lightbox
      navigation.ts                # Page router, terminal focus, connect panel
      theme.ts                      # 18 themes, auto mode, localStorage
      toast.ts                      # Toast notification queue
      presets.ts                    # Command presets CRUD
      connectionHistory.ts          # Saved devices (ip+port+method)
    views/
      DashboardView.vue            # Main dashboard layout
      LogcatView.vue               # Logcat streaming + filter UI
      AppsView.vue                  # App manager page
      ScreenshotsView.vue           # Screenshot gallery page
      SettingsView.vue              # Settings page
    components/
      TitleBar.vue                  # Custom window title bar
      AppSidebarLeft.vue            # Navigation sidebar (5 pages)
      AppSidebarRight.vue           # Controls sidebar (toggles, actions)
      ConnectPanel.vue              # Slide-in connection panel
      DeviceStatsCard.vue           # Battery, CPU, RAM, storage bar
      MirrorCard.vue                # Scrcpy mirror controls
      TerminalCard.vue              # ADB command terminal
      AppToast.vue                  # Toast notification overlay
      ScreenshotGrid.vue            # Screenshot thumbnail grid with lazy loading
      ScreenshotLightbox.vue        # Full-screen lightbox viewer
      settings/
        BinaryPathsCard.vue         # ADB/scrcpy path config + download
        GeneralPrefsCard.vue        # Auto-connect, theme, polling interval
        DisplayRecordingCard.vue    # Video quality, recording format, save dirs
    composables/
      useKeyboardShortcuts.ts       # Global keyboard shortcut wiring
      useLogcatChannel.ts           # Logcat store to Tauri stream bridge
      useApkDropZone.ts             # APK drag-and-drop composable
      useDropdownRegistry.ts        # Global dropdown coordination
      useImageLazy.ts               # IntersectionObserver lazy image loading
  src-tauri/                         # Rust backend (~4300 LOC)
    src/
      main.rs                        # Entry point
      lib.rs                         # AppState definition, command registration
      adb.rs                         # All adb_* command implementations (~2600 LOC)
      scrcpy.rs                      # Scrcpy launch/stop/find (~250 LOC)
      settings.rs                    # Binary validation, auto-detect, download (~600 LOC)
      icons.rs                       # App icon extraction engine (~680 LOC)
    tauri.conf.json                  # Window + bundle config
    capabilities/
      default.json                  # Permission grants
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

| Shortcut        | Action                       |
| --------------- | ---------------------------- |
| `Ctrl+K`        | Focus terminal input         |
| `Ctrl+M`        | Toggle screen mirror         |
| `Ctrl+Shift+S`  | Navigate to Screenshots page |
| `Ctrl+Shift+D`  | Disconnect device            |
| `Ctrl+,`        | Navigate to Settings page    |
| `Escape`        | Close menus / reboot menu    |
| `Alt+1`-`Alt+9` | Run command preset 1-9       |

---

## Progress Roadmap

> Done | In Progress | Planned

### Phase 1 - Core Foundation [DONE]

| #   | Feature                           | Status | Notes                                                       |
| --- | --------------------------------- | ------ | ----------------------------------------------------------- |
| 1.1 | Tauri v2 + Vue 3 project scaffold | Done   | Vite, Pinia, Tailwind v4                                    |
| 1.2 | Custom title bar                  | Done   | Traffic-light controls, drag region                         |
| 1.3 | ADB connect/disconnect            | Done   | WiFi, USB, pairing mode                                     |
| 1.4 | Device stats polling              | Done   | Battery, CPU, model info, RAM, storage, resolution, voltage |
| 1.5 | Connectivity toggles              | Done   | WiFi, data, airplane, Bluetooth                             |
| 1.6 | Navigation shortcuts              | Done   | Keyboard nav, terminal focus                                |

### Phase 2 - Controls and Terminal [DONE]

| #   | Feature              | Status | Notes                                           |
| --- | -------------------- | ------ | ----------------------------------------------- |
| 2.1 | Remote control keys  | Done   | Home, Back, Recent, Volume, Media, Power        |
| 2.2 | Text input to device | Done   | ADB input text                                  |
| 2.3 | Brightness control   | Done   | Slider 0-255                                    |
| 2.4 | DPI/density changer  | Done   | Slider + reset button                           |
| 2.5 | Terminal (ADB shell) | Done   | Timestamped logs, copy, export                  |
| 2.6 | System actions       | Done   | Screenshot, reboot (normal/recovery/bootloader) |
| 2.7 | Developer toggles    | Done   | Show taps, layout bounds, stay awake            |

### Phase 3 - Settings and Persistence [DONE]

| #   | Feature                | Status | Notes                                        |
| --- | ---------------------- | ------ | -------------------------------------------- |
| 3.1 | Settings persistence   | Done   | Tauri store plugin (JSON files)              |
| 3.2 | Binary path management | Done   | Auto-detect, manual, in-app ADB download     |
| 3.3 | Scrcpy configuration   | Done   | Quality presets, recording format, save dirs |
| 3.4 | Connection history     | Done   | Saved devices (ip+port+method)               |
| 3.5 | Multi-theme system     | Done   | 18 themes + auto mode                        |

### Phase 4 - Screen Mirroring [DONE]

| #   | Feature                 | Status | Notes                                                                |
| --- | ----------------------- | ------ | -------------------------------------------------------------------- |
| 4.1 | Scrcpy launch/stop      | Done   | Binary detection, process management                                 |
| 4.2 | Mirror control toggles  | Done   | Show touches, stay awake, turn screen off, always-on-top, no-control |
| 4.3 | Screen recording        | Done   | MKV/MP4 format, start/stop from UI                                   |
| 4.4 | Reconnect on disconnect | Done   | Auto-reconnect watcher                                               |

### Phase 5 - Logcat [DONE]

| #   | Feature                 | Status | Notes                                  |
| --- | ----------------------- | ------ | -------------------------------------- |
| 5.1 | Real-time logcat stream | Done   | Tauri event channel, colored output    |
| 5.2 | Level filtering         | Done   | Verbose/Debug/Info/Warning/Error/Fatal |
| 5.3 | Tag + text search       | Done   | Multi-tag include/exclude chips        |
| 5.4 | Active app filter       | Done   | Foreground package detection           |
| 5.5 | Pause/resume + export   | Done   | Stream control and file export         |

### Phase 6 - App Manager [DONE]

| #   | Feature                   | Status | Notes                                                   |
| --- | ------------------------- | ------ | ------------------------------------------------------- |
| 6.1 | App list with icons       | Done   | Grid view, sort, filter (system/user/all)               |
| 6.2 | App detail panel          | Done   | Version, dates, sizes, data/cache usage                 |
| 6.3 | App actions               | Done   | Open, uninstall, clear data, force stop, enable/disable |
| 6.4 | Icon extraction           | Done   | resources.arsc parser then on-device ZIP fallback       |
| 6.5 | Drag-and-drop APK install | Done   | Drop zone overlay, install queue                        |
| 6.6 | Keyboard navigation       | Done   | Arrow keys, Enter, Space, /, Escape                     |
| 6.7 | Active app detection      | Done   | Poll foreground package, badge in list                  |

### Phase 7 - Command Presets [DONE]

| #   | Feature             | Status | Notes                                       |
| --- | ------------------- | ------ | ------------------------------------------- |
| 7.1 | Preset CRUD         | Done   | Built-in + user presets, create/edit/delete |
| 7.2 | Quick-run shortcuts | Done   | Alt+1-9 keyboard shortcuts                  |
| 7.3 | Preset panel        | Done   | Sidebar with scroll, custom scrollbar       |

### Phase 8 - Screenshot Gallery [DONE]

| #   | Feature                 | Status | Notes                                                                                                           |
| --- | ----------------------- | ------ | --------------------------------------------------------------------------------------------------------------- |
| 8.1 | Screenshot gallery grid | Done   | Lazy-loaded thumbnails, intersection observer                                                                   |
| 8.2 | Sort and filter         | Done   | Newest/Oldest/Largest, All/Today/This Week                                                                      |
| 8.3 | Lightbox viewer         | Done   | Full-screen, arrow key navigation, zoom, metadata                                                               |
| 8.4 | Lightbox actions        | Done   | Open in viewer, open folder, copy path, delete                                                                  |
| 8.5 | Take and auto-refresh   | Done   | Capture button, debounced refresh, prepend path                                                                 |
| 8.6 | Rust backend            | Done   | adb_list_screenshots, adb_delete_screenshot, adb_read_file_base64, adb_get_file_info, PNG IHDR dimension parser |
| 8.7 | Keyboard shortcut       | Done   | Ctrl+Shift+S to Screenshots page                                                                                |

### Phase 9 - Polish and UX [DONE]

| #   | Feature             | Status | Notes                                           |
| --- | ------------------- | ------ | ----------------------------------------------- |
| 9.1 | Theme system        | Done   | 18 themes (6 dark, 6 mono, 6 light) + auto      |
| 9.2 | Theme-aware accents | Done   | Emerald accent, semantic token system           |
| 9.3 | Transition system   | Done   | Page transitions, stagger, reduced-motion       |
| 9.4 | Glass-morphism      | Done   | glass-card, backdrop-blur, border tokens        |
| 9.5 | Custom scrollbar    | Done   | Thin, theme-aware                               |
| 9.6 | Pressable buttons   | Done   | btn-pressable with scale transform              |
| 9.7 | Lightbox bug fix    | Done   | v-if instead of v-show prevents empty-src error |

---

### Phase 10 - Upcoming Features [PLANNED]

| #     | Feature                      | Priority | Notes                                         |
| ----- | ---------------------------- | -------- | --------------------------------------------- |
| 10.1  | File push/pull               | P1       | Push files to device, pull files from device  |
| 10.2  | Screen recording in-gallery  | P1       | View recordings alongside screenshots         |
| 10.3  | Batch screenshot operations  | P2       | Multi-select delete, share, copy paths        |
| 10.4  | Screenshot crop/annotate     | P2       | Basic crop + drawing overlay before share     |
| 10.5  | App backup and restore       | P2       | adb backup / adb restore with progress        |
| 10.6  | Device file explorer         | P2       | Browse device filesystem (limited to /sdcard) |
| 10.7  | Logcat bookmark/favorite     | P3       | Save filter presets per tag/level combo       |
| 10.8  | Custom command macro builder | P3       | Chain multiple ADB commands as a macro        |
| 10.9  | Multi-device support         | P3       | Switch between connected devices              |
| 10.10 | Auto-update                  | P3       | Tauri updater for new releases                |

### Phase 11 - Infrastructure and Quality [PLANNED]

| #    | Feature                   | Priority | Notes                                           |
| ---- | ------------------------- | -------- | ----------------------------------------------- |
| 11.1 | E2E tests (Playwright)    | P1       | Core flows: connect, command, screenshot        |
| 11.2 | Rust unit tests           | P1       | ADB command helpers, icon parser                |
| 11.3 | CI pipeline hardening     | P2       | Lint, type-check, build on all platforms        |
| 11.4 | Error boundary components | P2       | Graceful error UI instead of blank screen       |
| 11.5 | Accessibility audit       | P2       | ARIA labels, keyboard nav, focus trapping       |
| 11.6 | Performance profiling     | P3       | Large device lists, icon loading, logcat memory |

---

## License

This project is proprietary software. All rights reserved.

This project is proprietary software. All rights reserved.
