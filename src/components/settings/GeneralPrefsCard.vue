<script setup lang="ts">
import { useSettingsStore } from "../../stores/settings";
import { useThemeStore } from "../../stores/theme";
import { Settings } from "lucide-vue-next";

const store = useSettingsStore();
const themeStore = useThemeStore();

const pollingOptions = [1, 2, 3, 5, 10, 15, 30];
</script>

<template>
  <section class="card-glass border border-card-border rounded-2xl p-4">
    <div class="flex items-center gap-2 mb-4">
      <Settings :size="16" class="text-accent-emerald" />
      <h2 class="text-xs font-bold uppercase tracking-widest">General</h2>
    </div>

    <div class="space-y-5">
      <!-- Theme -->
      <div>
        <label class="text-sm block mb-2">Theme</label>
        <select v-model="themeStore.theme" @change="themeStore.setTheme(themeStore.theme)"
          class="w-full bg-black/40 border border-white/10 rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-emerald/50 cursor-pointer">
          <option value="dark" class="bg-gray-900">Dark</option>
          <option value="light" class="bg-gray-900">Light</option>
          <option value="system" class="bg-gray-900">System</option>
        </select>
      </div>

      <!-- Stay on Top -->
      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Stay on Top</span>
          <p class="text-[10px] text-gray-500">Keep window above other windows</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input type="checkbox" class="sr-only peer" v-model="store.stayOnTop"
            @change="store.setStayOnTop(store.stayOnTop)" />
          <div class="w-9 h-5 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-accent-emerald"></div>
        </label>
      </div>

      <!-- Auto-connect on Launch -->
      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Auto-connect on Launch</span>
          <p class="text-[10px] text-gray-500">Automatically detect and connect to devices on startup</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input type="checkbox" class="sr-only peer" v-model="store.autoConnectOnLaunch"
            @change="store.setAutoConnectOnLaunch(store.autoConnectOnLaunch)" />
          <div class="w-9 h-5 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-accent-emerald"></div>
        </label>
      </div>

      <!-- Auto-detect Binaries -->
      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Auto-detect Binaries</span>
          <p class="text-[10px] text-gray-500">Scan PATH for adb and scrcpy on startup</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input type="checkbox" class="sr-only peer" v-model="store.autoDetectBinaries"
            @change="store.setAutoDetectBinaries(store.autoDetectBinaries)" />
          <div class="w-9 h-5 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-accent-emerald"></div>
        </label>
      </div>

      <!-- Polling Interval -->
      <div>
        <label class="text-sm block mb-2">Device Stats Polling Interval</label>
        <select v-model.number="store.pollingInterval" @change="store.setPollingInterval(store.pollingInterval)"
          class="w-full bg-black/40 border border-white/10 rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-emerald/50 cursor-pointer">
          <option v-for="opt in pollingOptions" :key="opt" :value="opt" class="bg-gray-900">
            {{ opt }} second{{ opt > 1 ? 's' : '' }}
          </option>
        </select>
      </div>
    </div>
  </section>
</template>