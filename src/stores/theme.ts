import { defineStore } from 'pinia';
import { ref } from 'vue';

export const THEME_KEY = 'adb-powerhub-theme';

export type ThemePreference = 'dark' | 'light' | 'system';

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<ThemePreference>('dark');

  function init() {
    theme.value = 'dark';
    document.documentElement.setAttribute('data-theme', 'dark');
    localStorage.setItem(THEME_KEY, 'dark');
  }

  function setTheme(pref: ThemePreference) {
    theme.value = pref;
    localStorage.setItem(THEME_KEY, pref);
    document.documentElement.setAttribute('data-theme', 'dark');
  }

  return { theme, init, setTheme };
});
