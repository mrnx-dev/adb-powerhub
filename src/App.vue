<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useDeviceStore } from './stores/device';
import { useSettingsStore } from './stores/settings';
import { useNavigationStore } from './stores/navigation';
import { useKeyboardShortcuts } from './composables/useKeyboardShortcuts';
import { useConnectionHistoryStore } from './stores/connectionHistory';
import { useDropdownRegistry } from './composables/useDropdownRegistry';
import TitleBar from './components/TitleBar.vue';
import AppSidebarLeft from './components/AppSidebarLeft.vue';
import AppSidebarRight from './components/AppSidebarRight.vue';
import DashboardView from './views/DashboardView.vue';
import LogcatView from './views/LogcatView.vue';
import SettingsView from './views/SettingsView.vue';
import AppToast from './components/AppToast.vue';
import ConnectPanel from './components/ConnectPanel.vue';

import { useThemeStore } from './stores/theme';
import { usePresetsStore } from './stores/presets';

const deviceStore = useDeviceStore();
const settingsStore = useSettingsStore();
const navStore = useNavigationStore();
const themeStore = useThemeStore();
const connectionHistoryStore = useConnectionHistoryStore();
const presetsStore = usePresetsStore();

useKeyboardShortcuts();

// Page transition (FR-4) — map currentPage → component for <component :is> wrapper
const currentViewComponent = computed(() => {
  switch (navStore.currentPage) {
    case 'logcat':
      return LogcatView;
    case 'settings':
      return SettingsView;
    case 'dashboard':
    default:
      return DashboardView;
  }
});

// P25/M25: close all open Teleport-mounted dropdowns on view switch
// (e.g. DeviceStatsCard badge dropdown). The dropdown's own leave
// transition (100ms) completes within the first half of the page leave
// (150ms), so the dropdown disappears gracefully before the page does.
const dropdowns = useDropdownRegistry();
watch(
  () => navStore.currentPage,
  () => {
    if (dropdowns.state.openDropdowns.size > 0) {
      dropdowns.closeAll();
    }
  }
);

// Focus management (B4 fix) — after <Transition mode="out-in"> completes,
// move focus to new view's root so keyboard Tab navigation works smoothly.
const activeViewRef = ref<HTMLElement | null>(null);

function focusNewView(el: unknown) {
  if (el && el instanceof HTMLElement) {
    activeViewRef.value = el;
    nextTick(() => {
      el.focus({ preventScroll: true });
    });
  }
}

onMounted(async () => {
  themeStore.init();
  await settingsStore.init();
  await connectionHistoryStore.init();
  await presetsStore.loadPresets();

  if (settingsStore.autoConnectOnLaunch && settingsStore.adbValid) {
    await deviceStore.autoConnect();
  }

  await listen('scrcpy-exited', () => {
    deviceStore.mirroring = false;
    deviceStore.addLog('scrcpy window closed', 'info');
  });
});
</script>

<template>
  <div class="font-sans h-screen overflow-hidden flex flex-col bg-theme-page text-theme-primary">
    <TitleBar />
    <main class="flex flex-1 overflow-hidden">
      <AppSidebarLeft />
      <div class="flex-1 flex flex-col min-h-0 overflow-hidden relative">
        <Transition name="page-switch" mode="out-in">
          <component
            :is="currentViewComponent"
            :key="navStore.currentPage"
            @vue:mounted="focusNewView"
          />
        </Transition>
      </div>
      <AppSidebarRight />
    </main>
    <AppToast />
    <ConnectPanel />
  </div>
</template>

<style scoped>
/* Ensure view root can receive focus (B3 + B4 fix) */
:deep([tabindex='-1']) {
  outline: none;
}
</style>
