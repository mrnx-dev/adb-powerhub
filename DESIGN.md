# ADB PowerHub — Design System

> Living design reference for the ADB PowerHub desktop app.
> Stack: Vue 3 + Tailwind CSS v4 + Tauri v2.

---

## 1. Design Philosophy

**ADB PowerHub** is a dense, tool-focused desktop application for Android developers. The UI must feel:

- **Controlled and precise** — every toggle, stat, and log line is scannable.
- **Dark-first, theme-rich** — default is a dark glass-morphism shell, but users can switch between 18 curated themes.
- **Responsive to input** — buttons press, toggles snap, dropdowns originate from triggers.
- **Accessible by default** — keyboard navigation, focus traps, and `prefers-reduced-motion` are first-class.

The design language avoids decoration without purpose. Color is used for **state, hierarchy, and feedback**, not ornament.

---

## 2. Design Tokens

All visual values are exposed as CSS custom properties in `src/style.css`. Components should use the semantic tokens, not hardcoded colors.

### 2.1 Background Tokens

| Token | Usage |
|-------|-------|
| `--bg-page` | Main window background, radial glow edge |
| `--bg-sidebar` | Collapsible left/right sidebars |
| `--bg-card` | Cards, panels, settings sections |
| `--bg-terminal` | Terminal/log output areas |
| `--bg-input` | Text inputs, selects |
| `--bg-button` | Default button/resting surface |
| `--bg-hover` | Hover state for list items and buttons |
| `--bg-toggle-track` | Toggle switch / range slider track |

### 2.2 Foreground Tokens

| Token | Usage |
|-------|-------|
| `--text-primary` | Headings, primary labels, body text |
| `--text-secondary` | Descriptions, meta text |
| `--text-muted` | Placeholders, disabled hints, dividers |
| `--text-heading` | Section headings |
| `--text-inverse` | Text on top of accent/primary surfaces |

### 2.3 Border Tokens

| Token | Usage |
|-------|-------|
| `--border-primary` | Accent-tinted borders (cards, active states) |
| `--border-secondary` | Dividers, input borders, separators |
| `--border-tertiary` | Subtle button/resting borders |

### 2.4 Semantic Color Tokens

| Token | Meaning |
|-------|---------|
| `--color-primary` | Main accent (theme-specific) |
| `--color-primary-container` | Low-opacity accent background |
| `--color-on-primary` | Text/icons on primary surfaces |
| `--color-success` | Success / connected / ON state |
| `--color-success-container` | Soft success background |
| `--color-error` | Error / disconnect / recording indicator |
| `--color-error-container` | Soft error background |
| `--color-warning` | Warning / missing binary |
| `--color-warning-container` | Soft warning background |
| `--color-info` | Neutral info |
| `--color-info-container` | Soft info background |

### 2.5 Accent Opacity Utilities

Because Tailwind opacity variants generate static hex fallbacks, the app uses `color-mix()` utilities that adapt to the active theme:

| Class | Opacity |
|-------|---------|
| `bg-accent-subtle` | 5% |
| `bg-accent-light` | 10% |
| `bg-accent-default` | 20% |
| `bg-accent-medium` | 25% |
| `border-accent-light` | 10% |
| `border-accent-default` | 25% |
| `border-accent-strong` | 40% |
| `border-accent-focus` | 50% |

> Use these instead of `bg-accent-emerald/10` or similar static Tailwind utilities.

### 2.6 Typography

- **Font family:** `Inter`, `system-ui`, sans-serif (`--font-sans`)
- **Mono:** system monospace for terminal / logs
- **Scale:**
  - Page title: `text-sm font-semibold tracking-wider uppercase`
  - Card title: `text-xs font-semibold tracking-wider uppercase`
  - Body: `text-xs` / `text-sm`
  - Meta / labels: `text-[10px]`
  - Micro badges: `text-[9px]`
- **Body copy:** `user-select: none` globally (desktop app), except terminal/logs which use `select-text`.

### 2.7 Spacing & Radius

Border-radius scale (defined in `@theme`):

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | 4px | Tags, chips, micro badges |
| `--radius` | 6px | Small buttons, tags |
| `--radius-md` | 8px | **Default:** buttons, inputs, cards |
| `--radius-lg` | 12px | Panels, prominent cards |
| `--radius-xl` | 16px | Modals, dialogs |
| `--radius-2xl` | 20px | Decorative containers |
| `--radius-full` | 9999px | Pills, toggles, avatars |

Spacing follows Tailwind's scale. Cards use `p-4` or `p-6`. Sidebar gutters use `px-2` / `px-3` / `px-4` depending on expanded state.

### 2.8 Shadows

| Token | Usage |
|-------|-------|
| `--shadow-card` | Cards at rest |
| `--shadow-modal` | Slide panels, dropdowns, dialogs |
| `--shadow-hover` | Elevated hover surfaces |
| `--shadow-focus-ring` | Focus outline (3px accent glow) |

### 2.9 Scrollbar

Thin, theme-aware scrollbar:

- Width: `6px` vertical, `3px` horizontal (`.scrollbar-thin`)
- Track: `--scrollbar-track`
- Thumb: `--scrollbar-thumb` / `--scrollbar-thumb-hover`
- Applied globally via `::-webkit-scrollbar`.

---

## 3. Theme System

ADB PowerHub ships with **18 themes** plus an **Auto** mode that follows the OS color scheme.

### 3.1 Theme Catalog

**Dark themes (12):**

| ID | Name | Accent |
|----|------|--------|
| `emerald-night` | Emerald Night | `#10b981` |
| `ocean` | Ocean | `#22d3ee` |
| `sunset` | Sunset | `#f97316` |
| `sakura` | Sakura | `#f472b6` |
| `nord` | Nord | `#88c0d0` |
| `dracula` | Dracula | `#bd93f9` |
| `monokai` | Monokai | `#e6db58` |
| `solarized-dark` | Solarized Dark | `#268bd2` |
| `one-dark` | One Dark | `#61afef` |
| `tokyo-night` | Tokyo Night | `#7aa2f7` |
| `catppuccin-mocha` | Catppuccin Mocha | `#cba6f7` |
| `rose-pine` | Rosé Pine | `#c4a7e7` |
| `gruvbox-dark` | Gruvbox Dark | `#fe8019` |
| `cyberpunk` | Cyberpunk | `#05d9e8` |
| `high-contrast` | High Contrast | `#4ade80` |

**Light themes (3):**

| ID | Name | Accent |
|----|------|--------|
| `emerald-dawn` | Emerald Dawn | `#10b981` |
| `lavender-mist` | Lavender Mist | `#8b5cf6` |
| `solarized-light` | Solarized Light | `#268bd2` |

### 3.2 How Themes Work

- Theme is applied by setting `data-theme="{id}"` on `<html>`.
- Source of truth: `src/stores/theme.ts`.
- Persisted to Tauri store file `theme.json`.
- Auto mode listens to `prefers-color-scheme` and switches between the user's chosen dark and light themes.

### 3.3 Adding a New Theme

1. Add theme metadata to `THEME_CATALOG` in `src/stores/theme.ts`.
2. Add a matching `[data-theme='new-id']` block in `src/style.css` defining every token.
3. Keep the same token surface as existing themes for consistency.

---

## 4. Layout Architecture

### 4.1 App Shell

```
┌────────────────────────────┐
│ TitleBar (custom controls) │
├──────┬──────────────┬──────┤
│ Left │   Main View  │ Right│
│ Side-│  (router)    │ Side-│
│ bar  │              │ bar  │
├──────┴──────────────┴──────┤
│ Toast stack (fixed)        │
└────────────────────────────┘
```

- Left sidebar: navigation between pages (collapsible, 48px → 224px).
- Right sidebar: device controls (collapsible, 48px → 192px).
- Main area: single view rendered via `<Transition mode="out-in">`.
- Custom title bar with traffic-light window controls.
- `data-tauri-drag-region` marks draggable areas.

### 4.2 View Layouts

| View | Layout |
|------|--------|
| Dashboard | Vertical stack of cards (`DeviceStatsCard`, `MirrorCard`, `TerminalCard`) with stagger animation |
| Logcat | Full-width filter bar + scrollable stream |
| Apps | 3-panel layout: app list, detail, actions sidebar. Stacks vertically below `900px` |
| Screenshots | Filter bar + thumbnail grid + lightbox overlay |
| Files | File tree + breadcrumb + batch bar |
| Settings | Vertical stack of settings cards |

### 4.3 Glass Cards

Primary content container is `.card-glass`:

```css
.card-glass {
  background: var(--bg-card);
  backdrop-filter: blur(4px);
  border: 2px solid var(--border-primary);
  border-radius: var(--radius-md);
  box-shadow: 0 0 20px var(--glow-center);
}
```

Cards should not be nested inside other cards. Use `gap-6` between sibling cards.

### 4.4 Background Glow

The page background uses `.bg-glow`, a radial gradient from `--glow-center` to `--glow-edge` to create subtle depth behind cards.

---

## 5. Component Patterns

### 5.1 Buttons

Three main button styles:

| Style | Class | Use Case |
|-------|-------|----------|
| Primary | `.btn-primary` | Default buttons; hover fills with accent and inverts text |
| Accent / CTA | `.bg-accent-light` + `.hover-accent` | Send, execute, connect actions |
| Action Card | `.action-card` | Toggle cards with icon + label |

All pressable elements should include `.btn-pressable` for `:active` scale feedback.

### 5.2 Toggles

Use the custom switch markup (sr-only checkbox + styled div). Available in two sizes:

- Small (right sidebar): `w-8 h-[18px]` with `14px` knob
- Standard (settings): `w-9 h-5` with `16px` knob

Active state uses `--color-accent-emerald` for the track and knob.

### 5.3 Status Badges

| State | Class |
|-------|-------|
| Idle | `.status-badge-idle` |
| Active / Connected | `.status-badge-active` |
| Recording | `.status-badge-recording` |

Status dots pulse for active states and blink for recording.

### 5.4 Inputs

- Standard input: `bg-theme-input border border-theme-secondary rounded-lg px-4 py-2`
- Focus: `focus:border-accent-focus`
- Terminal input: bottom-border-only style (`.input-terminal`)
- Selects: custom arrow SVG, options styled with `--bg-card`
- Range slider: custom thumb/track using `--bg-toggle-track` and `--color-primary`

### 5.5 Progress Bars

Use the `.progress-bar-track` / `.progress-bar-fill` pattern:

```html
<div class="progress-bar-track">
  <div class="progress-bar-fill" :style="{ width: percent + '%' }"></div>
</div>
```

Height is always `2px`, with rounded caps.

### 5.6 Toast Notifications

- Position: fixed bottom-right, max-width `360px`
- Types: `success`, `error`, `info`, `progress`
- Includes colored left border and semantic icon
- Progress toasts show a fill bar
- Click-to-dismiss with scale-out

### 5.7 Dropdowns

- Teleport to `<body>` to escape stacking contexts
- Compute `transform-origin` from trigger rect (origin-aware animation)
- Enter: scale + fade from trigger origin
- Close on click outside, scroll, or Escape
- Register in `useDropdownRegistry` for global coordination

### 5.8 Slide Panels

Used for `ConnectPanel`. Slides in from the right over a backdrop blur.

- Panel width: `340px`
- Includes focus trap and Escape-to-close
- Backdrop click closes panel

---

## 6. Motion & Animation

### 6.1 Easing Curves

| Token | Curve | Use Case |
|-------|-------|----------|
| `--ease-out` | `cubic-bezier(0.23, 1, 0.32, 1)` | Default exits, hovers |
| `--ease-in-out` | `cubic-bezier(0.4, 0, 0.6, 1)` | Symmetric transitions |
| `--ease-emphasized` | `cubic-bezier(0.05, 0.7, 0.1, 1)` | Page switches, emphasized enters |
| `--ease-accelerate` | `cubic-bezier(0.3, 0, 1, 1)` | Quick exits |
| `--ease-bounce` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | Toggle knob, playful drops |

### 6.2 Duration Scale

| Token | Value | Use Case |
|-------|-------|----------|
| `--duration-quick` | 120ms | Press feedback, micro interactions |
| `--duration-standard` | 200ms | Hovers, fades, dropdowns |
| `--duration-card` | 250ms | Card transitions |
| `--duration-slow` | 300ms | Page transitions, panels |

### 6.3 Defined Transitions

| Name | File | Effect |
|------|------|--------|
| `page-switch` | `style.css` | View switch: opacity + 4px slide |
| `card-stagger` | `style.css` | Dashboard cards fade up with `--stagger-index` delay |
| `slide-panel` | `style.css` | Panel slides from right |
| `dropdown` | component scoped | Origin-aware scale + fade |
| `drop-zone` | `style.css` | APK drag overlay scale + fade |
| `toast` | `AppToast.vue` | Slide in/out from right |
| `status-fade` / `card-fade` | `style.css` | Connect panel micro transitions |
| `list-stagger` | `style.css` | Saved device list stagger |

### 6.4 Press Feedback

- `.btn-pressable:active:not(:disabled)` scales to `var(--scale-press)` (`0.97`)
- Transition uses `--duration-quick`
- Disabled elements do not scale

### 6.5 Reduced Motion

`@media (prefers-reduced-motion: reduce)` globally:

- Zeroes or reduces durations
- Disables transforms on page/card transitions
- Stops shimmer, pulse, and bounce animations
- Keeps opacity fades at 100–150ms for minimal feedback

---

## 7. Icons & Imagery

### 7.1 Icon Libraries

- **Lucide Vue** (`@lucide/vue`) — primary icon set, used for UI controls and navigation.
- **Material Design Icons** (`@mdi/font`) — available for legacy/fallback use.

### 7.2 Icon Sizing Conventions

| Size | Use |
|------|-----|
| `10px` | Inline micro actions |
| `12px` | Buttons, badges, small meta |
| `14px` | List items, card headers |
| `16px` | Section headings, nav items |
| `18px` | Sidebar toggle buttons |

### 7.3 App Icons / Thumbnails

- App icons in the app manager are extracted via Rust (`src-tauri/src/icons.rs`) with a fallback skeleton shimmer.
- Screenshot thumbnails lazy-load with `IntersectionObserver`.

---

## 8. Accessibility

### 8.1 Keyboard Navigation

- Global shortcuts handled in `useKeyboardShortcuts.ts`.
- Sidebar items, cards, and buttons are focusable and actionable.
- `Escape` closes panels, dropdowns, and reboot menus.
- Connect panel implements focus trap.
- After page switch, focus is moved to the new view root (`tabindex="-1"`).

### 8.2 Focus States

- Inputs: border-color transition to `--border-accent-focus`
- Buttons: rely on hover + active feedback
- Keyboard focus lists: `.keyboard-focus` applies `--bg-hover`

### 8.3 Reduced Motion

See [Motion §6.5](#65-reduced-motion). All animations degrade gracefully.

### 8.4 Contrast

- High-contrast theme (`high-contrast`) provides stronger borders and white-on-black text.
- Semantic colors maintain distinguishable containers even in light themes.

---

## 9. File Conventions

| Location | Purpose |
|----------|---------|
| `src/style.css` | All design tokens, utilities, animations, theme blocks |
| `src/stores/theme.ts` | Theme catalog, selection logic, persistence |
| `src/components/` | Reusable UI components |
| `src/views/` | Top-level page layouts |
| `src/composables/` | Shared behavior: keyboard, dropdowns, drag-drop, lazy load |
| `src-tauri/src/icons.rs` | Rust icon extraction engine |

### 9.1 Naming Conventions

- Theme-aware utility classes: `bg-theme-*`, `text-theme-*`, `border-theme-*`
- Semantic utilities: `bg-color-*`, `text-color-*`, `border-color-*`
- Accent opacity utilities: `bg-accent-*`, `border-accent-*`, `hover-bg-accent-*`, `hover-border-accent-*`
- Component classes: `card-glass`, `action-card`, `toggle-pill`, `status-badge`, `btn-pressable`

---

## 10. Do's and Don'ts

### ✅ Do

- Use semantic tokens (`var(--bg-card)`, `var(--text-primary)`) instead of hardcoded colors.
- Apply `.btn-pressable` to all clickable surfaces.
- Wrap page-level cards in `.card-glass` with consistent `p-4` / `gap-6` spacing.
- Respect `prefers-reduced-motion` when adding new animations.
- Teleport dropdowns to `<body>` and register them in `useDropdownRegistry`.
- Use the accent `color-mix()` utilities for opacity variants.

### ❌ Don't

- Hardcode hex colors in components — always go through tokens.
- Use `transition: all 300ms` or generic `ease-in` on UI elements.
- Nest cards inside cards.
- Animate layout-triggering properties (width/height) without `will-change` containment.
- Forget `:active` feedback on buttons.
- Skip focus management in modals, panels, and new views.

---

## 11. Changelog

| Date | Change |
|------|--------|
| 2026-06-15 | Initial DESIGN.md — consolidated tokens, themes, layout, components, and motion from `src/style.css` and core components. |
