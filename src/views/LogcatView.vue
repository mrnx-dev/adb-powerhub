<script setup lang="ts">
import { onMounted, watch } from 'vue';
import { useDeviceStore } from '../stores/device';
import { useLogcatStore } from '../stores/logcat';
import { useLogcatChannel } from '../composables/useLogcatChannel';
import LogcatToolbar from '../components/LogcatToolbar.vue';
import LogcatFilterBar from '../components/LogcatFilterBar.vue';
import LogcatStream from '../components/LogcatStream.vue';

const deviceStore = useDeviceStore();
const store = useLogcatStore();
const channel = useLogcatChannel();

onMounted(() => {
  if (deviceStore.connected && !store.streaming) {
    channel.start();
  }
});

watch(
  () => store.streaming,
  (isStreaming) => {
    if (!isStreaming) {
      store.stopActiveAppPolling();
    } else if (store.activeAppOnly) {
      store.startActiveAppPolling();
    }
  }
);

watch(
  () => deviceStore.connected,
  (connected) => {
    if (!connected) {
      store.setActiveAppOnly(false);
      if (store.streaming) {
        store.status = 'DISCONNECTED';
        channel.stop();
      }
    }
  }
);

// Handle Start requests from toolbar (store-based command)
watch(
  () => store.startRequested,
  (requested) => {
    if (!requested) return;
    store.startRequested = false;
    if (deviceStore.connected && !store.streaming) {
      channel.start();
    }
  }
);

// Handle Refresh/Restart requests from toolbar (store-based command)
watch(
  () => store.restartRequested,
  async (requested) => {
    if (!requested) return;
    store.restartRequested = false;
    if (!deviceStore.connected) return;
    store.restarting = true;
    try {
      await channel.restart();
    } finally {
      store.restarting = false;
    }
  }
);
</script>

<template>
  <div class="flex-1 min-h-0 overflow-hidden bg-glow p-6 flex flex-col gap-4">
    <div
      v-if="!deviceStore.connected"
      class="flex-1 flex flex-col items-center justify-center gap-4"
    >
      <div class="text-theme-muted">No device connected. Connect a device to stream logs.</div>
      <button
        class="px-4 py-2 rounded-lg bg-accent-emerald/15 border border-accent-emerald/30 text-accent-emerald text-sm font-medium hover:bg-accent-emerald/25 transition-all"
        @click="deviceStore.autoConnect()"
      >
        Reconnect
      </button>
    </div>

    <template v-else>
      <LogcatToolbar />
      <LogcatFilterBar />
      <LogcatStream class="flex-1 min-h-0" />
    </template>
  </div>
</template>
