<script setup lang="ts">
import DeviceStatsCard from '../components/DeviceStatsCard.vue';
import MirrorCard from '../components/MirrorCard.vue';
import TerminalCard from '../components/TerminalCard.vue';
import { useDeviceStore } from '../stores/device';
import { Loader2, X } from '@lucide/vue';

const deviceStore = useDeviceStore();
</script>

<template>
  <div tabindex="-1" class="flex-1 min-h-0 overflow-hidden bg-glow p-6 flex flex-col gap-6">
    <!-- Reconnecting indicator -->
    <div
      v-if="!deviceStore.connected && deviceStore.isReconnecting"
      class="flex items-center gap-2.5 py-2.5 px-3 rounded-lg bg-accent-light border border-accent-default"
    >
      <Loader2 :size="14" class="text-accent-emerald animate-spin shrink-0" />
      <div class="flex-1 min-w-0">
        <div class="text-xs font-semibold text-accent-emerald">Reconnecting...</div>
        <div class="text-[10px] text-theme-muted">
          Attempt {{ deviceStore.reconnectAttempt }}/30
        </div>
      </div>
      <button
        class="p-1.5 rounded-md text-theme-muted hover:text-color-error hover:bg-color-error-container transition-colors"
        title="Cancel reconnect"
        @click="deviceStore.stopReconnectWatcher()"
      >
        <X :size="14" />
      </button>
    </div>
    <TransitionGroup
      tag="div"
      name="card-stagger"
      appear
      class="flex flex-col gap-6 flex-1 min-h-0"
    >
      <DeviceStatsCard key="device-stats" :style="{ '--stagger-index': 0 }" />
      <MirrorCard key="mirror" :style="{ '--stagger-index': 1 }" />
      <TerminalCard key="terminal" :style="{ '--stagger-index': 2 }" />
    </TransitionGroup>
  </div>
</template>
