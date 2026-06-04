<script setup lang="ts">
import { computed } from 'vue';
import { useSettingsStore } from '../../stores/settings';
import { useDeviceStore } from '../../stores/device';
import { useThemeStore } from '../../stores/theme';
import { THEME_CATALOG, DARK_THEMES, LIGHT_THEMES } from '../../stores/theme';
import type { ThemeId, ThemeMode } from '../../stores/theme';
import { Settings, Palette } from '@lucide/vue';

const store = useSettingsStore();
const deviceStore = useDeviceStore();
const themeStore = useThemeStore();

const pollingOptions = [1, 2, 3, 5, 10, 15, 30];

const modes: { value: ThemeMode; label: string }[] = [
  { value: 'dark', label: 'Dark' },
  { value: 'light', label: 'Light' },
  { value: 'auto', label: 'Auto' },
];

const visibleThemes = computed(() => {
  if (themeStore.mode === 'dark') return DARK_THEMES;
  if (themeStore.mode === 'light') return LIGHT_THEMES;
  return THEME_CATALOG; // auto: show both groups
});

const darkGroup = computed(() => DARK_THEMES);
const lightGroup = computed(() => LIGHT_THEMES);

function isActiveTheme(id: ThemeId): boolean {
  if (themeStore.mode === 'dark') return id === themeStore.darkThemeId;
  if (themeStore.mode === 'light') return id === themeStore.lightThemeId;
  // auto: highlight both selected dark and light
  return id === themeStore.darkThemeId || id === themeStore.lightThemeId;
}

function selectTheme(id: ThemeId) {
  const theme = THEME_CATALOG.find((t) => t.id === id);
  if (!theme) return;
  if (theme.type === 'dark') {
    themeStore.setDarkTheme(id);
  } else {
    themeStore.setLightTheme(id);
  }
}

function handleAutoReconnectChange(val: boolean) {
  store.setAutoReconnect(val);
  if (!val) {
    deviceStore.stopReconnectWatcher();
  }
}
</script>

<template>
  <section class="card-glass p-4">
    <div class="flex items-center gap-2 mb-4">
      <Settings :size="16" class="text-accent-emerald" />
      <h2 class="font-sans text-xs font-semibold tracking-wider uppercase">General</h2>
    </div>

    <div class="space-y-5">
      <!-- Theme Section -->
      <div>
        <div class="flex items-center gap-2 mb-3">
          <Palette :size="14" class="text-accent-emerald" />
          <span class="text-sm font-medium">Theme</span>
        </div>

        <!-- Mode pills -->
        <div class="flex gap-1 mb-3">
          <button
            v-for="m in modes"
            :key="m.value"
            class="btn-pressable flex-1 px-3 py-1.5 rounded-lg text-xs font-medium border transition-all"
            :class="
              themeStore.mode === m.value
                ? 'bg-accent-10 border-accent-25 text-accent-emerald'
                : 'bg-theme-btn border-theme-tertiary text-theme-secondary hover:border-theme-secondary'
            "
            @click="themeStore.setMode(m.value)"
          >
            {{ m.label }}
          </button>
        </div>

        <!-- Auto mode: Dark themes group -->
        <template v-if="themeStore.mode === 'auto'">
          <p class="text-[10px] text-theme-muted mb-1.5 font-medium uppercase tracking-wider">
            Dark themes
          </p>
          <div class="grid grid-cols-3 gap-2 mb-2">
            <button
              v-for="theme in darkGroup"
              :key="theme.id"
              class="btn-pressable flex items-center gap-2 px-2.5 py-2 rounded-lg border text-xs transition-all"
              :class="
                theme.id === themeStore.darkThemeId
                  ? 'bg-accent-10 border-accent-25 text-theme-primary'
                  : 'bg-theme-btn border-theme-tertiary text-theme-secondary hover:border-theme-secondary'
              "
              @click="themeStore.setDarkTheme(theme.id)"
            >
              <span
                class="w-3.5 h-3.5 rounded-full shrink-0 border border-white/10"
                :style="{ backgroundColor: theme.accent }"
              />
              <span class="truncate">{{ theme.name }}</span>
            </button>
          </div>

          <p class="text-[10px] text-theme-muted mb-1.5 font-medium uppercase tracking-wider">
            Light themes
          </p>
          <div class="grid grid-cols-3 gap-2 mb-2">
            <button
              v-for="theme in lightGroup"
              :key="theme.id"
              class="btn-pressable flex items-center gap-2 px-2.5 py-2 rounded-lg border text-xs transition-all"
              :class="
                theme.id === themeStore.lightThemeId
                  ? 'bg-accent-10 border-accent-25 text-theme-primary'
                  : 'bg-theme-btn border-theme-tertiary text-theme-secondary hover:border-theme-secondary'
              "
              @click="themeStore.setLightTheme(theme.id)"
            >
              <span
                class="w-3.5 h-3.5 rounded-full shrink-0 border border-white/10"
                :style="{ backgroundColor: theme.accent }"
              />
              <span class="truncate">{{ theme.name }}</span>
            </button>
          </div>

          <p class="text-[10px] text-theme-muted mt-1">
            Auto switches between your dark and light theme based on OS appearance.
          </p>
        </template>

        <!-- Dark or Light mode: single grid -->
        <template v-else>
          <div class="grid grid-cols-3 gap-2 mb-1">
            <button
              v-for="theme in visibleThemes"
              :key="theme.id"
              class="btn-pressable flex items-center gap-2 px-2.5 py-2 rounded-lg border text-xs transition-all"
              :class="
                isActiveTheme(theme.id)
                  ? 'bg-accent-10 border-accent-25 text-theme-primary'
                  : 'bg-theme-btn border-theme-tertiary text-theme-secondary hover:border-theme-secondary'
              "
              @click="selectTheme(theme.id)"
            >
              <span
                class="w-3.5 h-3.5 rounded-full shrink-0 border border-white/10"
                :style="{ backgroundColor: theme.accent }"
              />
              <span class="truncate">{{ theme.name }}</span>
            </button>
          </div>
        </template>
      </div>

      <!-- Stay on Top -->
      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Stay on Top</span>
          <p class="text-[10px] text-theme-muted">Keep window above other windows</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer select-none">
          <input
            v-model="store.stayOnTop"
            type="checkbox"
            class="sr-only peer"
            @change="store.setStayOnTop(store.stayOnTop)"
          />
          <div
            class="w-9 h-5 bg-theme-toggle-track rounded-full peer peer-focus:outline-none after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[var(--bg-page)] after:rounded-full after:h-4 after:w-4 after:shadow-md after:transition-all after:duration-300 after:ease-[cubic-bezier(0.34,1.56,0.64,1)] peer-checked:after:translate-x-[14px] rtl:peer-checked:after:-translate-x-[14px] peer-checked:after:bg-accent-emerald peer-active:after:w-[22px]"
          ></div>
        </label>
      </div>

      <!-- Auto-detect Binaries -->
      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Auto-detect Binaries</span>
          <p class="text-[10px] text-theme-muted">Scan PATH for adb and scrcpy on startup</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer select-none">
          <input
            v-model="store.autoDetectBinaries"
            type="checkbox"
            class="sr-only peer"
            @change="store.setAutoDetectBinaries(store.autoDetectBinaries)"
          />
          <div
            class="w-9 h-5 bg-theme-toggle-track rounded-full peer peer-focus:outline-none peer-checked:bg-accent-emerald after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[var(--bg-page)] after:rounded-full after:h-4 after:w-4 after:shadow-md after:transition-all after:duration-300 after:ease-[cubic-bezier(0.34,1.56,0.64,1)] peer-checked:after:translate-x-[14px] rtl:peer-checked:after:-translate-x-[14px] peer-checked:after:bg-accent-emerald peer-active:after:w-[22px]"
          ></div>
        </label>
      </div>

      <!-- Auto-Reconnect -->
      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Auto-Reconnect</span>
          <p class="text-[10px] text-theme-muted">Reconnect automatically when device comes back</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer select-none">
          <input
            v-model="store.autoReconnect"
            type="checkbox"
            class="sr-only peer"
            @change="handleAutoReconnectChange(store.autoReconnect)"
          />
          <div
            class="w-9 h-5 bg-theme-toggle-track rounded-full peer peer-focus:outline-none peer-checked:bg-accent-emerald after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[var(--bg-page)] after:rounded-full after:h-4 after:w-4 after:shadow-md after:transition-all after:duration-300 after:ease-[cubic-bezier(0.34,1.56,0.64,1)] peer-checked:after:translate-x-[14px] rtl:peer-checked:after:-translate-x-[14px] peer-checked:after:bg-accent-emerald peer-active:after:w-[22px]"
          ></div>
        </label>
      </div>

      <!-- Polling Interval -->
      <div>
        <label class="text-sm block mb-2">Device Stats Polling Interval</label>
        <select
          v-model.number="store.pollingInterval"
          class="w-full bg-theme-input border border-theme-secondary rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-50 cursor-pointer hover:border-accent-30"
          @change="store.setPollingInterval(store.pollingInterval)"
        >
          <option v-for="opt in pollingOptions" :key="opt" :value="opt" class="bg-theme-card">
            {{ opt }} second{{ opt > 1 ? 's' : '' }}
          </option>
        </select>
      </div>
    </div>
  </section>
</template>
