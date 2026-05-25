<script setup lang="ts">
import { useDeviceStore } from "../stores/device";
import { useSettingsStore } from "../stores/settings";
import { useNavigationStore } from "../stores/navigation";
import { MonitorPlay, Settings } from "lucide-vue-next";

const store = useDeviceStore();
const settings = useSettingsStore();
const nav = useNavigationStore();

const showScrcpyWarning = !settings.scrcpyValid && !store.scrcpyAvailable;

function toggleMirror() {
  if (store.mirroring) {
    store.stopMirror();
  } else {
    store.launchMirror();
  }
}
</script>

<template>
  <section class="card-glass border border-card-border rounded-2xl p-4">
    <div class="flex items-center gap-2 mb-3">
      <MonitorPlay :size="14" class="text-accent-emerald" />
      <h2 class="text-xs font-bold uppercase tracking-widest">Mirror</h2>
    </div>

    <div v-if="showScrcpyWarning" class="text-[10px] text-yellow-500 bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-2 mb-3">
      <template v-if="settings.scrcpyPath && !settings.scrcpyValid">
        Configured scrcpy path is invalid.
      </template>
      <template v-else>
        scrcpy not found. Install it or set the path in Settings.
      </template>
      <button @click="nav.navigateTo('settings')"
        class="ml-1 underline hover:text-yellow-300 inline-flex items-center gap-1">
        <Settings :size="10" /> Settings
      </button>
    </div>

    <div class="flex items-center gap-2 mb-3">
      <button @click="toggleMirror" :disabled="!store.connected"
        class="bg-accent-emerald hover:bg-accent-emerald-hover text-white px-4 py-2 rounded-lg text-xs font-semibold transition-colors shadow-lg shadow-emerald-500/20 disabled:opacity-50 disabled:cursor-not-allowed">
        {{ store.mirroring ? "Stop" : "Start" }}
      </button>
      <label class="flex items-center gap-2 cursor-pointer group">
        <input type="checkbox" v-model="store.recordingScreen"
          class="w-4 h-4 rounded border-white/10 bg-black/40 text-accent-emerald focus:ring-accent-emerald" />
        <span class="text-[10px] text-gray-300 group-hover:text-white transition-colors">Record</span>
      </label>
    </div>

    <div class="flex gap-2">
      <button @click="store.toggleStayAwake(!store.stayAwakeEnabled)"
        class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg transition-all text-[10px] font-medium"
        :class="store.stayAwakeEnabled ? 'bg-accent-emerald/10 border border-accent-emerald/50 text-accent-emerald' : 'bg-white/5 border border-white/5 hover:border-accent-emerald/50 hover:bg-white/10 text-gray-300'">
        <span class="mdi mdi-white-balance-sunny text-xs"></span>
        Stay Awake
      </button>

      <button @click="store.rotateDevice()"
        class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-white/5 border border-white/5 hover:border-accent-emerald/50 hover:bg-white/10 transition-all text-[10px] font-medium text-gray-300">
        <span class="mdi mdi-screen-rotation text-xs"></span>
        Rotate
      </button>
    </div>
  </section>
</template>