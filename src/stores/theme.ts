import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { load, type Store } from '@tauri-apps/plugin-store';

// ─── Theme ID Union (18 string literals) ────────────────────
export type ThemeId =
  | 'emerald-night'
  | 'emerald-dawn'
  | 'ocean'
  | 'sunset'
  | 'sakura'
  | 'nord'
  | 'dracula'
  | 'monokai'
  | 'lavender-mist'
  | 'high-contrast'
  | 'solarized-dark'
  | 'solarized-light'
  | 'one-dark'
  | 'tokyo-night'
  | 'catppuccin-mocha'
  | 'rose-pine'
  | 'gruvbox-dark'
  | 'cyberpunk';

export type ThemeMode = 'dark' | 'light' | 'auto';

// ─── Theme Metadata ─────────────────────────────────────────
export interface ThemeMeta {
  id: ThemeId;
  name: string;
  type: 'dark' | 'light';
  accent: string;
}

// ─── Theme Catalog (static, 18 entries) ──────────────────────
export const THEME_CATALOG: ThemeMeta[] = [
  { id: 'emerald-night', name: 'Emerald Night', type: 'dark', accent: '#10b981' },
  { id: 'emerald-dawn', name: 'Emerald Dawn', type: 'light', accent: '#10b981' },
  { id: 'ocean', name: 'Ocean', type: 'dark', accent: '#22d3ee' },
  { id: 'sunset', name: 'Sunset', type: 'dark', accent: '#f97316' },
  { id: 'sakura', name: 'Sakura', type: 'dark', accent: '#f472b6' },
  { id: 'nord', name: 'Nord', type: 'dark', accent: '#88c0d0' },
  { id: 'dracula', name: 'Dracula', type: 'dark', accent: '#bd93f9' },
  { id: 'monokai', name: 'Monokai', type: 'dark', accent: '#e6db58' },
  { id: 'lavender-mist', name: 'Lavender Mist', type: 'light', accent: '#8b5cf6' },
  { id: 'high-contrast', name: 'High Contrast', type: 'dark', accent: '#4ade80' },
  { id: 'solarized-dark', name: 'Solarized Dark', type: 'dark', accent: '#268bd2' },
  { id: 'solarized-light', name: 'Solarized Light', type: 'light', accent: '#268bd2' },
  { id: 'one-dark', name: 'One Dark', type: 'dark', accent: '#61afef' },
  { id: 'tokyo-night', name: 'Tokyo Night', type: 'dark', accent: '#7aa2f7' },
  { id: 'catppuccin-mocha', name: 'Catppuccin Mocha', type: 'dark', accent: '#cba6f7' },
  { id: 'rose-pine', name: 'Rosé Pine', type: 'dark', accent: '#c4a7e7' },
  { id: 'gruvbox-dark', name: 'Gruvbox Dark', type: 'dark', accent: '#fe8019' },
  { id: 'cyberpunk', name: 'Cyberpunk', type: 'dark', accent: '#05d9e8' },
];

export const DARK_THEMES = THEME_CATALOG.filter((t) => t.type === 'dark');
export const LIGHT_THEMES = THEME_CATALOG.filter((t) => t.type === 'light');

// ─── Valid ID sets for quick lookup ──────────────────────────
const DARK_IDS = new Set<ThemeId>(DARK_THEMES.map((t) => t.id));
const LIGHT_IDS = new Set<ThemeId>(LIGHT_THEMES.map((t) => t.id));
// ─── Tauri Store Instance (own file: theme.json) ────────────
let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load('theme.json', { autoSave: true, defaults: {} });
  }
  return storeInstance;
}

// ─── Store ───────────────────────────────────────────────────
export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>('auto');
  const darkThemeId = ref<ThemeId>('emerald-night');
  const lightThemeId = ref<ThemeId>('emerald-dawn');
  const activeThemeId = ref<ThemeId>('emerald-night');

  // Media query listener refs
  let mediaQuery: MediaQueryList | null = null;
  let mediaHandler: ((e: MediaQueryListEvent) => void) | null = null;

  // ─── Validate theme ID — fall back to defaults ──────────
  function validateThemeId(id: ThemeId, type: 'dark' | 'light'): ThemeId {
    const validSet = type === 'dark' ? DARK_IDS : LIGHT_IDS;
    if (validSet.has(id)) return id;
    return type === 'dark' ? 'emerald-night' : 'emerald-dawn';
  }

  // ─── Resolve activeThemeId from mode + OS preference ─────
  function resolve() {
    if (mode.value === 'auto') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      activeThemeId.value = prefersDark ? darkThemeId.value : lightThemeId.value;
    } else if (mode.value === 'dark') {
      activeThemeId.value = darkThemeId.value;
    } else {
      activeThemeId.value = lightThemeId.value;
    }
  }

  // ─── Apply theme to DOM ──────────────────────────────────
  function apply(themeId: ThemeId) {
    document.documentElement.setAttribute('data-theme', themeId);
  }

  // ─── Persist 3 keys to theme.json ────────────────────────
  async function persist() {
    try {
      const s = await getStore();
      await s.set('themeMode', mode.value);
      await s.set('themeDarkId', darkThemeId.value);
      await s.set('themeLightId', lightThemeId.value);
    } catch (e) {
      console.error('Failed to persist theme:', e);
    }
  }

  // ─── Load from theme.json, fallback to defaults ──────────
  async function loadFromSettings() {
    try {
      const s = await getStore();
      const savedMode = await s.get<string>('themeMode');
      const savedDarkId = await s.get<string>('themeDarkId');
      const savedLightId = await s.get<string>('themeLightId');

      if (savedMode && ['dark', 'light', 'auto'].includes(savedMode)) {
        mode.value = savedMode as ThemeMode;
      }
      if (savedDarkId && DARK_IDS.has(savedDarkId as ThemeId)) {
        darkThemeId.value = savedDarkId as ThemeId;
      }
      if (savedLightId && LIGHT_IDS.has(savedLightId as ThemeId)) {
        lightThemeId.value = savedLightId as ThemeId;
      }
    } catch {
      // First run, no saved theme — defaults apply
    }
  }

  // ─── Attach OS media listener (mode=auto) ────────────────
  function attachMediaListener() {
    if (mediaQuery) return; // idempotent
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaHandler = () => {
      resolve();
      apply(activeThemeId.value);
    };
    mediaQuery.addEventListener('change', mediaHandler);
  }

  // ─── Detach OS media listener ────────────────────────────
  function detachMediaListener() {
    if (mediaQuery && mediaHandler) {
      mediaQuery.removeEventListener('change', mediaHandler);
    }
    mediaQuery = null;
    mediaHandler = null;
  }

  // ─── Init (async — call with await from App.vue) ────────
  async function init() {
    // Clean up stale localStorage key from old implementation
    localStorage.removeItem('adb-powerhub-theme');

    await loadFromSettings();
    resolve();
    apply(activeThemeId.value);
    if (mode.value === 'auto') {
      attachMediaListener();
    }
  }

  // ─── Set mode ────────────────────────────────────────────
  async function setMode(newMode: ThemeMode) {
    mode.value = newMode;
    if (newMode === 'auto') {
      attachMediaListener();
    } else {
      detachMediaListener();
    }
    resolve();
    apply(activeThemeId.value);
    await persist();
  }

  // ─── Set dark theme ──────────────────────────────────────
  async function setDarkTheme(id: ThemeId) {
    darkThemeId.value = validateThemeId(id, 'dark');
    if (
      mode.value === 'dark' ||
      (mode.value === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches)
    ) {
      activeThemeId.value = darkThemeId.value;
      apply(activeThemeId.value);
    }
    await persist();
  }

  // ─── Set light theme ─────────────────────────────────────
  async function setLightTheme(id: ThemeId) {
    lightThemeId.value = validateThemeId(id, 'light');
    if (
      mode.value === 'light' ||
      (mode.value === 'auto' && !window.matchMedia('(prefers-color-scheme: dark)').matches)
    ) {
      activeThemeId.value = lightThemeId.value;
      apply(activeThemeId.value);
    }
    await persist();
  }

  // ─── Computed: theme meta objects ─────────────────────────
  const darkTheme = computed(
    () => THEME_CATALOG.find((t) => t.id === darkThemeId.value) ?? THEME_CATALOG[0]
  );
  const lightTheme = computed(
    () => THEME_CATALOG.find((t) => t.id === lightThemeId.value) ?? THEME_CATALOG[1]
  );
  const activeTheme = computed(
    () => THEME_CATALOG.find((t) => t.id === activeThemeId.value) ?? THEME_CATALOG[0]
  );

  return {
    mode,
    darkThemeId,
    lightThemeId,
    activeThemeId,
    darkTheme,
    lightTheme,
    activeTheme,
    init,
    setMode,
    setDarkTheme,
    setLightTheme,
  };
});
