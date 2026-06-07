<script setup lang="ts">
import { onMounted, watch } from 'vue';
import { useDeviceStore } from '../stores/device';
import { useAppsStore } from '../stores/apps';
import { useNavigationStore } from '../stores/navigation';
import { Download } from '@lucide/vue';
import { useApkDropZone } from '../composables/useApkDropZone';
import AppList from '../components/AppList.vue';
import AppPreview from '../components/AppPreview.vue';
import AppActions from '../components/AppActions.vue';

const FILTER_LABELS: Record<string, string> = {
  all: 'All',
  third_party: 'Third-Party',
  system: 'System',
  disabled: 'Disabled',
};

const FILTER_KEYS = ['all', 'third_party', 'system', 'disabled'] as const;

const deviceStore = useDeviceStore();
const appsStore = useAppsStore();
const navStore = useNavigationStore();
const { isDragOver } = useApkDropZone();

onMounted(() => {
  if (deviceStore.connected) {
    loadApps();
  }
});

async function loadApps() {
  await appsStore.fetchApps();
  // Don't await — icons load in background (with 1 retry built-in)
  appsStore.fetchIcons();
}

watch(
  () => deviceStore.connected,
  (connected) => {
    if (!connected) {
      appsStore.reset();
    }
  }
);

watch(
  () => navStore.currentPage,
  (page) => {
    if (page === 'apps' && deviceStore.connected) {
      loadApps();
    }
  }
);
</script>

<template>
  <div tabindex="-1" class="flex-1 min-h-0 overflow-hidden bg-glow p-6 flex flex-col gap-4">
    <!-- Disconnected state -->
    <div
      v-if="!deviceStore.connected"
      class="flex-1 flex flex-col items-center justify-center gap-4"
    >
      <div class="text-theme-muted">No device connected. Connect a device to manage apps.</div>
      <button
        class="px-4 py-2 rounded-lg bg-accent-light border border-accent-strong text-accent-emerald text-sm font-medium hover:bg-accent-medium btn-pressable"
        @click="deviceStore.autoConnect()"
      >
        Reconnect
      </button>
    </div>

    <!-- Connected state -->
    <template v-else>
      <!-- Toolbar -->
      <div class="flex items-center gap-3 flex-wrap">
        <h2 class="text-lg font-semibold text-theme-primary">
          Apps
          <span v-if="appsStore.appCount > 0" class="text-theme-muted text-sm font-normal ml-1">
            ({{ appsStore.appCount }})
          </span>
        </h2>

        <div class="flex gap-1 ml-2">
          <button
            v-for="f in FILTER_KEYS"
            :key="f"
            class="btn-pressable px-2.5 py-1 rounded-md text-xs font-medium border"
            :class="[
              appsStore.filter === f
                ? 'bg-accent-light border-accent-strong text-accent-emerald'
                : 'bg-theme-btn border-theme-tertiary text-theme-secondary hover-accent',
            ]"
            :aria-label="`Filter by ${FILTER_LABELS[f]}`"
            @click="
              appsStore.filter = f;
              loadApps();
            "
          >
            {{ FILTER_LABELS[f] }}
            <span class="ml-1 opacity-60">({{ appsStore.filterCounts[f] }})</span>
          </button>
        </div>

        <div class="flex-1" />

        <button
          class="btn-pressable px-3 py-1.5 rounded-lg bg-theme-btn border border-theme-tertiary text-theme-secondary text-xs font-medium hover-accent"
          :disabled="appsStore.isLoading"
          aria-label="Refresh app list"
          @click="loadApps()"
        >
          {{ appsStore.isLoading ? 'Loading...' : 'Refresh' }}
        </button>

        <button
          class="btn-pressable px-3 py-1.5 rounded-lg bg-accent-light border border-accent-strong text-accent-emerald text-xs font-medium hover:bg-accent-medium"
          :disabled="appsStore.isInstalling"
          aria-label="Install APK"
          @click="appsStore.installApk()"
        >
          {{ appsStore.isInstalling ? 'Installing...' : '+ Install APK' }}
        </button>
      </div>

      <!-- Drop zone hint (always visible, highlights during drag) -->
      <div
        class="rounded-lg border-2 border-dashed p-3 flex items-center justify-center gap-2 transition-colors duration-150 ease-out"
        :class="[
          isDragOver && deviceStore.connected
            ? 'drop-zone-hint-active'
            : 'border-theme-tertiary/50',
        ]"
      >
        <Download :size="16" class="text-theme-muted" />
        <span class="text-sm text-theme-muted">Drop APK here to install</span>
      </div>

      <!-- Error -->
      <div
        v-if="appsStore.error"
        class="px-4 py-2 rounded-lg bg-color-error-container border border-color-error text-color-error text-sm"
      >
        {{ appsStore.error }}
        <button class="btn-pressable ml-2 underline text-color-error" @click="loadApps()">
          Retry
        </button>
      </div>

      <!-- Loading skeleton -->
      <div
        v-if="appsStore.isLoading && appsStore.apps.length === 0"
        class="flex-1 flex items-center justify-center"
      >
        <div class="flex flex-col items-center gap-3">
          <div class="flex gap-2">
            <div class="w-2 h-2 rounded-full bg-theme-hover animate-pulse" />
            <div
              class="w-2 h-2 rounded-full bg-theme-hover animate-pulse"
              style="animation-delay: 150ms"
            />
            <div
              class="w-2 h-2 rounded-full bg-theme-hover animate-pulse"
              style="animation-delay: 300ms"
            />
          </div>
          <div class="text-theme-muted text-sm">Loading apps...</div>
        </div>
      </div>

      <!-- Main content: 3-panel -->
      <div v-else class="apps-layout flex-1 min-h-0 flex gap-0 overflow-hidden">
        <AppList class="w-[35%] min-w-[240px] rounded-l-lg" />
        <AppPreview class="w-[35%] min-w-[240px]" />
        <AppActions class="w-[30%] min-w-[200px]" />
      </div>
    </template>
  </div>
</template>
