<script setup lang="ts">
import { useDeviceStore } from "../stores/device";
import { useSettingsStore } from "../stores/settings";
import { useNavigationStore } from "../stores/navigation";
import { MonitorPlay, Settings, Square, Sun, RotateCw, AlertTriangle } from "lucide-vue-next";
import { computed } from "vue";

const store = useDeviceStore();
const settings = useSettingsStore();
const nav = useNavigationStore();

const showScrcpyWarning = !settings.scrcpyValid && !store.scrcpyAvailable;

const statusState = computed(() => {
  if (store.mirroring && store.recordingScreen) return "recording";
  if (store.mirroring) return "active";
  return "idle";
});

const statusLabel = computed(() => {
  switch (statusState.value) {
    case "recording": return "Recording";
    case "active": return "Mirroring";
    default: return "Idle";
  }
});

function toggleMirror() {
  if (store.mirroring) {
    store.stopMirror();
  } else {
    store.launchMirror();
  }
}
</script>

<template>
  <section class="card-glass p-4">
    <!-- Header -->
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-2">
        <MonitorPlay :size="16" class="text-accent-emerald" />
        <h2 class="font-sans text-xs font-semibold tracking-wider uppercase">Mirror</h2>
      </div>
      <div class="status-badge" :class="{
        'status-badge-idle': statusState === 'idle',
        'status-badge-active': statusState === 'active',
        'status-badge-recording': statusState === 'recording'
      }">
        <span class="status-dot" :class="{
          'status-dot-idle': statusState === 'idle',
          'status-dot-active': statusState === 'active',
          'status-dot-recording': statusState === 'recording'
        }"></span>
        {{ statusLabel }}
      </div>
    </div>

    <!-- scrcpy Warning -->
    <div v-if="showScrcpyWarning" class="flex items-start gap-2 text-[11px] text-color-warning bg-color-warning-container border border-color-warning rounded-lg p-2.5 mb-4">
      <AlertTriangle :size="14" class="shrink-0 mt-0.5" />
      <div class="flex-1">
        <template v-if="settings.scrcpyPath && !settings.scrcpyValid">
          Configured scrcpy path is invalid.
        </template>
        <template v-else>
          scrcpy not found. Install it or set the path in Settings.
        </template>
        <button @click="nav.navigateTo('settings')"
          class="ml-1 underline hover:text-color-warning inline-flex items-center gap-1">
          <Settings :size="10" /> Settings
        </button>
      </div>
    </div>

    <!-- Hero Button Area -->
    <div class="bg-theme-terminal rounded-lg p-5 mb-4">
      <div class="flex flex-col items-center gap-3">
        <button @click="toggleMirror" :disabled="!store.connected"
          class="w-full py-3.5 rounded-lg text-sm font-semibold transition-all duration-200 disabled:opacity-40 disabled:cursor-not-allowed flex items-center justify-center gap-2"
          :class="store.mirroring
            ? 'bg-accent-emerald text-white border border-accent-emerald hover:bg-accent-emerald-hover'
            : 'btn-primary border border-theme-secondary hover:border-accent-emerald'">
          <template v-if="store.mirroring">
            <Square :size="18" />
            <span>Stop Mirror</span>
          </template>
          <template v-else>
            <MonitorPlay :size="18" />
            <span>Start Mirror</span>
          </template>
        </button>

        <!-- Record Toggle Pill -->
        <div @click="store.recordingScreen = !store.recordingScreen"
          class="toggle-pill" :class="{ 'toggle-pill-on': store.recordingScreen }">
          <span class="toggle-pill-knob"></span>
          <span>{{ store.recordingScreen ? "Recording" : "Record" }}</span>
        </div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="flex items-center gap-2 text-[10px] text-theme-muted uppercase tracking-wider font-semibold mb-2">
      Quick Actions
    </div>
    <div class="grid grid-cols-2 gap-2">
      <button @click="store.toggleStayAwake(!store.stayAwakeEnabled)"
        class="action-card" :class="{ 'action-card-active': store.stayAwakeEnabled }">
        <Sun :size="15" />
        <span class="flex-1 text-left">Stay Awake</span>
        <span v-if="store.stayAwakeEnabled" class="text-[9px] uppercase font-bold opacity-70">ON</span>
      </button>

      <button @click="store.rotateDevice()"
        class="action-card">
        <RotateCw :size="15" />
        <span class="flex-1 text-left">Rotate</span>
      </button>
    </div>
  </section>
</template>