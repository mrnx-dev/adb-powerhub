<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
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
import AppsView from './views/AppsView.vue';
import SettingsView from './views/SettingsView.vue';
import AppToast from './components/AppToast.vue';
import ConnectPanel from './components/ConnectPanel.vue';

import { Package, WifiOff } from '@lucide/vue';
import { useThemeStore } from './stores/theme';
import { usePresetsStore } from './stores/presets';
import { useApkDropZone, initApkDropZone, destroyApkDropZone } from './composables/useApkDropZone';

const deviceStore = useDeviceStore();
const settingsStore = useSettingsStore();
const navStore = useNavigationStore();
const themeStore = useThemeStore();
const connectionHistoryStore = useConnectionHistoryStore();
const presetsStore = usePresetsStore();

const { isDragOver, hasApkFiles, queueProgress } = useApkDropZone();

useKeyboardShortcuts();

// Page transition (FR-4) — map currentPage → component for <component :is> wrapper
const currentViewComponent = computed(() => {
  switch (navStore.currentPage) {
    case 'logcat':
      return LogcatView;
    case 'apps':
      return AppsView;
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
  await initApkDropZone();
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

onBeforeUnmount(() => {
  destroyApkDropZone();
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

    <!-- APK Drag & Drop overlay -->
    <Transition name="drop-zone">
      <div
        v-if="isDragOver && hasApkFiles"
        class="fixed inset-0 z-50 flex items-center justify-center pointer-events-none"
      >
        <div class="drop-zone-overlay flex flex-col items-center gap-3 px-8 py-6">
          <template v-if="deviceStore.connected">
            <Package :size="48" class="text-accent-emerald drop-zone-icon" />
            <p class="text-lg font-semibold text-theme-primary">Drop APK to install</p>
            <p v-if="queueProgress" class="text-sm text-theme-muted">
              Installing {{ queueProgress.current }}/{{ queueProgress.total }}...
            </p>
          </template>
          <template v-else>
            <WifiOff :size="48" class="text-theme-muted" />
            <p class="text-lg font-semibold text-theme-muted">Connect a device first</p>
          </template>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* Ensure view root can receive focus (B3 + B4 fix) */
:deep([tabindex='-1']) {
  outline: none;
}
</style>
