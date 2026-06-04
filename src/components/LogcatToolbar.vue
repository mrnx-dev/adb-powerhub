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
        'bg-color-success-container border-color-success text-color-success':
          store.status === 'LIVE',
        'bg-color-warning-container border-color-warning text-color-warning':
          store.status === 'PAUSED',
        'bg-color-error-container border-color-error text-color-error':
          store.status === 'DISCONNECTED' || store.status === 'ERROR',
        'bg-theme-btn border-theme-tertiary text-theme-muted': store.status === 'IDLE',
      }"
    >
      <span
        class="w-1.5 h-1.5 rounded-full inline-block"
        :class="{
          'bg-[var(--color-success)]': store.status === 'LIVE',
          'bg-[var(--color-warning)]': store.status === 'PAUSED',
          'bg-[var(--color-error)]': store.status === 'DISCONNECTED' || store.status === 'ERROR',
          'bg-theme-muted': store.status === 'IDLE',
        }"
      ></span>
      {{ store.status }}
    </div>

    <div class="flex-1" />

    <!-- Refresh / Reconnect -->
    <button
      class="btn-pressable flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-theme-btn border border-theme-tertiary text-theme-secondary hover-accent"
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
      class="btn-pressable px-3 py-1.5 rounded-lg text-xs font-medium bg-accent-light border border-accent-default text-accent-emerald hover:bg-accent-default flex items-center gap-1.5"
      @click="store.requestStart()"
    >
      <Play :size="14" /> Start
    </button>

    <!-- Pause / Resume -->
    <button
      v-if="store.streaming"
      class="btn-pressable px-3 py-1.5 rounded-lg text-xs font-medium bg-theme-btn border border-theme-tertiary text-theme-secondary hover-accent flex items-center gap-1.5"
      @click="togglePause"
    >
      <Pause v-if="!store.paused" :size="14" />
      <Play v-else :size="14" />
      {{ store.paused ? 'Resume' : 'Pause' }}
    </button>

    <!-- Clear -->
    <button
      class="btn-pressable px-3 py-1.5 rounded-lg text-xs font-medium bg-theme-btn border border-theme-tertiary text-theme-secondary hover-accent flex items-center gap-1.5"
      :disabled="store.totalCount === 0 || !deviceStore.connected"
      :class="{ 'opacity-40 cursor-not-allowed': store.totalCount === 0 || !deviceStore.connected }"
      @click="clearBuffer"
    >
      <Trash2 :size="14" /> Clear
    </button>

    <!-- Export -->
    <button
      class="btn-pressable px-3 py-1.5 rounded-lg text-xs font-medium bg-theme-btn border border-theme-tertiary text-theme-secondary hover-accent flex items-center gap-1.5"
      :disabled="store.visibleCount === 0"
      :class="{ 'opacity-40 cursor-not-allowed': store.visibleCount === 0 }"
      @click="store.exportLogs()"
    >
      <Download :size="14" /> Export
    </button>
  </div>
</template>
