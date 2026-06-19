# AGENTS.md — Project Rules for AI Coding Agents

> Auto-discovered by: Pi, OpenCode, Claude Code, Cursor, Codex CLI, Amp, Droid
> Place at project root. Copy this file to your project root and customize it — keep the structure, swap the values.

---

## Project Overview

**Project:** ADB PowerHub
**Description:** Cross-platform desktop app for controlling Android devices via ADB, built with Tauri v2, Vue 3, and Rust.

---

## Tech Stack

- **Runtime:** Node.js 22+ (frontend tooling), Rust stable (Tauri backend)
- **Framework:** Vue 3 (Composition API + `<script setup>`) + Tauri v2
- **Language:** TypeScript (frontend), Rust 2021 edition (backend)
- **Database:** None — persistence via Tauri store plugin (JSON files)
- **ORM / Data Access:** None
- **Auth:** None
- **Styling:** Tailwind CSS v4 + custom CSS theme tokens
- **Icons:** Lucide Vue + Material Design Icons
- **Package Manager:** npm
- **Test Framework:** None yet (Rust unit tests and Playwright E2E planned)

---

## Quick Commands

```bash
npm run tauri dev            # Start Tauri dev window with Vite HMR
npm run tauri build          # Build production installers for current platform
npm run dev                  # Start Vite web dev server only (no Tauri window)
npm run build                # Build Vite frontend bundle only
npm run lint                 # ESLint with auto-fix + Prettier
npm run lint:check           # ESLint without fixing
npm run format               # Prettier format all files
npm run format:check         # Prettier check only
npm run test                # Run frontend unit tests (vitest, one-shot)
npm run test:watch          # Vitest watch mode for dev loop
npm run test:coverage       # Vitest with coverage report (local only)
npx vue-tsc --noEmit         # Type-check Vue/TypeScript frontend
cargo check --manifest-path src-tauri/Cargo.toml   # Type-check Rust backend
cargo test --manifest-path src-tauri/Cargo.toml    # Run Rust unit tests
cargo test --manifest-path src-tauri/Cargo.toml icons   # Run tests for one module
```

## Test & CI gating

- **Frontend tests:** Vitest + happy-dom + @vue/test-utils. Tests colocated in
  `src/**/__tests__/*.test.ts`. Run `npm run test` (one-shot) or
  `npm run test:watch` during dev. Stores that call Tauri `invoke` are tested
  by mocking `@tauri-apps/api/core` at the module boundary (`vi.mock`); stores
  that use `@tauri-apps/plugin-store` are tested with an in-memory fake `load()`.
- **Rust tests:** inline `#[cfg(test)] mod tests` per module + binary fixtures
  in `src-tauri/tests/fixtures/`. Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- **CI jobs:** `ci.yml` runs `test-rust` and `test-frontend` on ubuntu-22.04
  on every push/PR. They are currently in a **grace period** (non-required).
  After green and stable for 1-2 weeks, enable them as required:
  1. GitHub repo -> Settings -> Branches -> Add branch protection rule for `main`
  2. Tick "Require status checks to pass before merging"
  3. Add `test-rust` and `test-frontend` to the required list

  This step is manual and cannot be automated from a file.

- **Flaky policy:** quarantine via `#[ignore]` (Rust) or `test.skip` (Vitest)
  with a TODO referencing a follow-up issue. No auto-retry.

---

## Code Conventions

### Naming

- **Frontend:** camelCase for variables/functions/composables, PascalCase for Vue components and Pinia stores (e.g., `DeviceStatsCard.vue`, `useApkDropZone.ts`, `deviceStore`)
- **Backend Rust:** snake_case for modules, functions, variables; PascalCase for structs/enums/traits (e.g., `adb_connect`, `AppState`, `DeviceInfo`)
- **File names:** match exported symbol casing (`.vue` PascalCase, `.ts` camelCase)

### Imports

- Use `@/` alias for frontend imports (`import { useDeviceStore } from '@/stores/device'`)
- Prefer explicit imports from Tauri plugin APIs (`@tauri-apps/api/core`, `plugin-*`)
- No barrel exports/index files unless explicitly required

### Architecture Rules

- All ADB / shell / filesystem operations MUST go through Rust Tauri commands, never from frontend JS directly
- Frontend state lives in Pinia stores under `src/stores/`; UI behavior lives in composables under `src/composables/`
- Validate user-provided paths and IP:port inputs at the Rust boundary; reject unsafe patterns
- Use typed Tauri command payloads (`#[tauri::command]` + serde structs) instead of raw strings where possible

### Branching Protocol

Every feature implementation MUST happen on a new git branch or in a separate worktree. Helper:

```bash
./scripts/miko-branch.sh <feature-name>        # creates feat/YYYY-MM-DD-feature-name
./scripts/miko-branch.sh <feature-name> --worktree  # also creates a git worktree
```

If the helper is not available, create the branch manually:

```bash
git checkout -b feat/YYYY-MM-DD-feature-name
# or for a worktree
git worktree add ../project-feature-name feat/YYYY-MM-DD-feature-name
```

- Never implement directly on the default branch.
- Use sub-agents or worktrees to delegate implementation.
- Merge only after review passes.

### Database / Data Access

- No SQL database; persistence uses Tauri store plugin (`@tauri-apps/plugin-store`)
- Settings, presets, and connection history are stored as JSON files managed by their respective Pinia stores
- Never read/write files directly from frontend code; use Tauri commands or store plugin APIs

### State Management

- Client state: Pinia (`src/stores/`)
- Global UI coordination: custom composables (`useDropdownRegistry`, `useApkDropZone`, `useKeyboardShortcuts`)
- Theme state persists to `localStorage` and applies CSS theme class on `<html>`

### UI / Styling

- Tailwind CSS v4 utility classes with custom theme tokens defined in `src/style.css`
- Use semantic tokens (e.g., `bg-surface`, `text-primary`, `border-card`) over raw color values
- No inline `style` attributes except for truly dynamic values (sliders, progress bars)
- Custom glass-morphism cards use `glass-card` utility; buttons use `btn-pressable` for tactile feedback

---

## NEVER DO

- ❌ Never execute shell/ADB commands directly from frontend JavaScript — always route through Rust Tauri commands
- ❌ Never bypass Tauri capability permissions or add overly broad `shell`/`fs` scope without explicit review
- ❌ Never hardcode ADB/scrcpy binary paths; use settings store + auto-detect helpers
- ❌ Never store credentials, tokens, or secrets in `localStorage`/store JSON files — this app has no auth and should not gain any
- ❌ Never run blocking synchronous operations inside Tauri commands; use async/`tokio::spawn` for long-running processes (logcat, scrcpy, ADB shell)
- ❌ Never use `v-show` for components whose children fail when data is absent (gallery lightbox, app preview); prefer `v-if` with null guards

---

## Project Structure

```
adb-powerhub/
├── src/                         # Vue 3 frontend
│   ├── main.ts                  # Vue + Pinia bootstrap
│   ├── App.vue                  # Root layout, drop zone, page transitions
│   ├── style.css                # Tailwind v4 + theme tokens
│   ├── stores/                  # Pinia state modules (device, settings, logcat, ...)
│   ├── views/                   # Top-level page components
│   ├── components/              # Reusable UI components + page-specific subfolders
│   └── composables/             # Shared UI behavior hooks
├── src-tauri/                   # Rust Tauri backend
│   ├── src/                     # Rust source (commands, app state, utilities)
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Window, bundle, and capability config
├── public/                      # Static assets
├── scripts/                     # Miko branching helpers
├── skills/                      # Pi specialist skill definitions
└── dist/                        # Vite production output
```

---

## Key Files

| File                        | Purpose                                             |
| --------------------------- | --------------------------------------------------- |
| `src/main.ts`               | Vue app bootstrap + Pinia registration              |
| `src/App.vue`               | Root view router, APK drop zone, page transitions   |
| `src/style.css`             | Tailwind v4 import + custom theme token definitions |
| `src/stores/device.ts`      | ADB connection, polling, stats, commands            |
| `src/stores/settings.ts`    | Binary paths, preferences, Tauri store persistence  |
| `src-tauri/src/lib.rs`      | Tauri app state + command registration              |
| `src-tauri/src/adb.rs`      | Core ADB command implementations                    |
| `src-tauri/src/scrcpy.rs`   | Scrcpy launch/stop helpers                          |
| `src-tauri/src/settings.rs` | Binary validation, auto-detect, ADB download        |
| `src-tauri/tauri.conf.json` | Window config, permissions, bundle metadata         |
| `vite.config.js`            | Vite + Vue + Tailwind plugin config                 |
| `tsconfig.json`             | TypeScript paths (`@/*`) and strict checks          |
| `eslint.config.js`          | ESLint flat config for JS/TS/Vue                    |
| `.prettierrc`               | Prettier formatting rules                           |
| `.env.example`              | Environment variables reference                     |

---

## Development Pipeline (Optional)

If using Miko orchestrator, every new feature follows:

```
PRD → Blueprint → Implementation Plan → Execute → Review
```

- Each feature must be implemented on a new git branch or in a separate worktree
- Brainstorming must present 3–5 choosable options before proceeding
- See `ORCHESTRATOR.md` for full pipeline documentation.

## Skills (Optional)

Load specialist skills in Pi:

```bash
pi --skill ./skills/<skill-folder>
```

See `SOUL.md` for the team roster and trigger words.

---

## Notes for AI Agents

1. **Read this file first** — it contains project-specific rules you must follow.
2. **Read SOUL.md / ORCHESTRATOR.md** if available — they describe the development methodology.
3. **When in doubt, ask** — don't assume conventions from other stacks.
4. **Never hardcode stack assumptions** — if you see Vue/Tauri/Rust/npm in this file, it's because THIS project uses them, not because they're universal.
