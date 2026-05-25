<script setup lang="ts">
import { onMounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { useDeviceStore } from "./stores/device";
import { useSettingsStore } from "./stores/settings";
import { useNavigationStore } from "./stores/navigation";
import { useKeyboardShortcuts } from "./composables/useKeyboardShortcuts";
import TitleBar from "./components/TitleBar.vue";
import AppSidebarLeft from "./components/AppSidebarLeft.vue";
import AppSidebarRight from "./components/AppSidebarRight.vue";
import DashboardView from "./views/DashboardView.vue";
import SettingsView from "./views/SettingsView.vue";
import AppToast from "./components/AppToast.vue";

const deviceStore = useDeviceStore();
const settingsStore = useSettingsStore();
const navStore = useNavigationStore();

useKeyboardShortcuts();

onMounted(async () => {
  await settingsStore.init();

  if (settingsStore.autoConnectOnLaunch && settingsStore.adbValid) {
    deviceStore.autoConnect();
  }

  await listen("scrcpy-exited", () => {
    deviceStore.mirroring = false;
    deviceStore.addLog("scrcpy window closed", "info");
  });
});
</script>

<template>
  <div class="bg-app-dark text-gray-200 font-sans h-screen overflow-hidden flex flex-col">
    <TitleBar />
    <main class="flex flex-1 overflow-hidden">
      <AppSidebarLeft />
      <DashboardView v-if="navStore.currentPage === 'dashboard'" />
      <SettingsView v-else-if="navStore.currentPage === 'settings'" />
      <AppSidebarRight />
    </main>
    <AppToast />
  </div>
</template>