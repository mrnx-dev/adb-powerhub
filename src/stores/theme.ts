import { defineStore } from "pinia";
import { ref } from "vue";

export const THEME_KEY = "adb-powerhub-theme";

export type ThemePreference = "dark" | "light" | "system";

export const useThemeStore = defineStore("theme", () => {
  const theme = ref<ThemePreference>("dark");

  function getSystemTheme(): "dark" | "light" {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }

  function applyTheme(pref: ThemePreference) {
    const resolved = pref === "system" ? getSystemTheme() : pref;
    document.documentElement.setAttribute("data-theme", resolved);
    theme.value = pref;
    localStorage.setItem(THEME_KEY, pref);
  }

  function init() {
    const saved = localStorage.getItem(THEME_KEY) as ThemePreference | null;
    applyTheme(saved || "dark");

    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", (e) => {
        if (theme.value === "system") {
          document.documentElement.setAttribute("data-theme", e.matches ? "dark" : "light");
        }
      });
  }

  function setTheme(pref: ThemePreference) {
    applyTheme(pref);
  }

  return { theme, init, setTheme };
});
