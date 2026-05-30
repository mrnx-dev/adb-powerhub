<script setup lang="ts">
import { onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useDeviceStore } from './stores/device';
import { useSettingsStore } from './stores/settings';
import { useNavigationStore } from './stores/navigation';
import { useKeyboardShortcuts } from './composables/useKeyboardShortcuts';
import { useConnectionHistoryStore } from './stores/connectionHistory';
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
      <div class="flex-1 flex flex-col min-h-0 overflow-hidden">
        <DashboardView v-if="navStore.currentPage === 'dashboard'" />
        <LogcatView v-else-if="navStore.currentPage === 'logcat'" />
        <SettingsView v-else-if="navStore.currentPage === 'settings'" />
      </div>
      <AppSidebarRight />
    </main>
    <AppToast />
    <ConnectPanel />
  </div>
</template>

<style scoped></style>
