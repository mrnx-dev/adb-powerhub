<script setup lang="ts">
import { useLogcatStore } from '../stores/logcat';
import { useDeviceStore } from '../stores/device';
import { Pause, Play, Trash2, Download, ScrollText, RefreshCw } from '@lucide/vue';

const store = useLogcatStore();
const deviceStore = useDeviceStore();

async function togglePause() {
  store.setPaused(!store.paused);
}

async function clearBuffer() {
  try {
    store.clearLocalBuffer();
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('adb_clear_logcat_buffer');
  } catch (e) {
    store.error = String(e);
  }
}
</script>

<template>
  <div class="flex items-center gap-3 shrink-0">
    <!-- Title + Status -->
    <div class="flex items-center gap-2">
      <ScrollText :size="18" class="text-accent-emerald" />
      <span class="text-sm font-semibold text-theme-primary">Logcat</span>
    </div>

    <!-- Status Indicator -->
    <div
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border"
      :class="{
        'bg-emerald-500/10 border-emerald-500/25 text-emerald-400': store.status === 'LIVE',
        'bg-amber-500/10 border-amber-500/25 text-amber-400': store.status === 'PAUSED',
        'bg-red-500/10 border-red-500/25 text-red-400':
          store.status === 'DISCONNECTED' || store.status === 'ERROR',
        'bg-gray-500/10 border-gray-500/25 text-gray-400': store.status === 'IDLE',
      }"
    >
      <span
        class="w-1.5 h-1.5 rounded-full inline-block"
        :class="{
          'bg-emerald-400': store.status === 'LIVE',
          'bg-amber-400': store.status === 'PAUSED',
          'bg-red-400': store.status === 'DISCONNECTED' || store.status === 'ERROR',
          'bg-gray-400': store.status === 'IDLE',
        }"
      ></span>
      {{ store.status }}
    </div>

    <div class="flex-1" />

    <!-- Refresh / Reconnect -->
    <button
      class="px-3 py-1.5 rounded-lg text-xs font-medium bg-theme-btn border border-theme-tertiary text-theme-secondary hover-accent transition-all flex items-center gap-1.5"
      :disabled="!deviceStore.connected || store.restarting"
      :class="{ 'opacity-40 cursor-not-allowed': !deviceStore.connected || store.restarting }"
      @click="store.requestRestart()"
    >
      <RefreshCw :size="14" :class="{ 'animate-spin': store.restarting }" />
      {{ store.restarting ? 'Restarting…' : 'Refresh' }}
    </button>

    <!-- Start (if idle/disconnected) -->
    <button
      v-if="!store.streaming && deviceStore.connected"
      class="px-3 py-1.5 rounded-lg text-xs font-medium bg-accent-emerald/10 border border-accent-emerald/25 text-accent-emerald hover:bg-accent-emerald/20 transition-all flex items-center gap-1.5"
      @click="store.requestStart()"
    >
      <Play :size="14" /> Start
    </button>

    <!-- Pause / Resume -->
    <button
      v-if="store.streaming"
      class="px-3 py-1.5 rounded-lg text-xs font-medium bg-theme-btn border border-theme-tertiary text-theme-secondary hover-accent transition-all flex items-center gap-1.5"
      @click="togglePause"
    >
      <Pause v-if="!store.paused" :size="14" />
      <Play v-else :size="14" />
      {{ store.paused ? 'Resume' : 'Pause' }}
    </button>

    <!-- Clear -->
    <button
      class="px-3 py-1.5 rounded-lg text-xs font-medium bg-theme-btn border border-theme-tertiary text-theme-secondary hover-accent transition-all flex items-center gap-1.5"
      :disabled="store.totalCount === 0 || !deviceStore.connected"
      :class="{ 'opacity-40 cursor-not-allowed': store.totalCount === 0 || !deviceStore.connected }"
      @click="clearBuffer"
    >
      <Trash2 :size="14" /> Clear
    </button>

    <!-- Export -->
    <button
      class="px-3 py-1.5 rounded-lg text-xs font-medium bg-theme-btn border border-theme-tertiary text-theme-secondary hover-accent transition-all flex items-center gap-1.5"
      :disabled="store.visibleCount === 0"
      :class="{ 'opacity-40 cursor-not-allowed': store.visibleCount === 0 }"
      @click="store.exportLogs()"
    >
      <Download :size="14" /> Export
    </button>
  </div>
</template>
