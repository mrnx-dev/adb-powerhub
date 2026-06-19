import { describe, it, expect, vi, beforeEach } from 'vitest';

const fakeData = new Map<string, unknown>();
const fakeStore = {
  get: vi.fn((key: string) => Promise.resolve(fakeData.get(key))),
  set: vi.fn((key: string, value: unknown) => {
    fakeData.set(key, value);
    return Promise.resolve();
  }),
  save: vi.fn(() => Promise.resolve()),
};

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(() => Promise.resolve(fakeStore)),
}));

// matchMedia is not implemented in happy-dom — stub it.
let prefersDark = false;
const matchMediaMock = vi.fn((query: string) => ({
  matches: query.includes('dark') ? prefersDark : false,
  media: query,
  onchange: null,
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  addListener: vi.fn(),
  removeListener: vi.fn(),
  dispatchEvent: vi.fn(() => false),
}));

import { useThemeStore } from '../theme';

describe('useThemeStore', () => {
  beforeEach(() => {
    fakeData.clear();
    prefersDark = false;
    matchMediaMock.mockClear();
    vi.stubGlobal('matchMedia', matchMediaMock);
    document.documentElement.removeAttribute('data-theme');
  });

  it('setMode("dark") activates the dark theme id on <html data-theme>', async () => {
    const store = useThemeStore();
    await store.setMode('dark');
    expect(store.mode).toBe('dark');
    expect(store.activeThemeId).toBe(store.darkThemeId);
    expect(document.documentElement.getAttribute('data-theme')).toBe(store.darkThemeId);
  });

  it('setMode("light") activates the light theme id', async () => {
    const store = useThemeStore();
    await store.setMode('light');
    expect(store.mode).toBe('light');
    expect(store.activeThemeId).toBe(store.lightThemeId);
    expect(document.documentElement.getAttribute('data-theme')).toBe(store.lightThemeId);
  });

  it('setMode("auto") resolves via OS preference (prefers dark)', async () => {
    prefersDark = true;
    const store = useThemeStore();
    await store.setMode('auto');
    expect(store.mode).toBe('auto');
    expect(store.activeThemeId).toBe(store.darkThemeId);
  });

  it('setDarkTheme updates the dark slot and applies when mode=dark', async () => {
    const store = useThemeStore();
    await store.setMode('dark');
    await store.setDarkTheme('ocean');
    expect(store.darkThemeId).toBe('ocean');
    expect(store.activeThemeId).toBe('ocean');
    expect(document.documentElement.getAttribute('data-theme')).toBe('ocean');
  });

  it('setDarkTheme rejects a light-only theme id and falls back to default', async () => {
    const store = useThemeStore();
    await store.setMode('dark');
    // "emerald-dawn" is a LIGHT theme — invalid for the dark slot
    await store.setDarkTheme('emerald-dawn');
    expect(store.darkThemeId).toBe('emerald-night'); // fallback to default
  });

  it('persist writes themeMode/themeDarkId/themeLightId to the store', async () => {
    const store = useThemeStore();
    await store.setMode('dark');
    await store.setDarkTheme('ocean');
    await store.setLightTheme('lavender-mist');

    expect(fakeData.get('themeMode')).toBe('dark');
    expect(fakeData.get('themeDarkId')).toBe('ocean');
    expect(fakeData.get('themeLightId')).toBe('lavender-mist');
  });
});
