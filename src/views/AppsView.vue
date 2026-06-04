<script setup lang="ts">
import { onMounted, watch } from 'vue';
import { useDeviceStore } from '../stores/device';
import { useAppsStore } from '../stores/apps';
import { useNavigationStore } from '../stores/navigation';
import AppList from '../components/AppList.vue';
import AppDetail from '../components/AppDetail.vue';

const deviceStore = useDeviceStore();
const appsStore = useAppsStore();
const navStore = useNavigationStore();

onMounted(() => {
  if (deviceStore.connected) {
    appsStore.fetchApps();
  }
});

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
      appsStore.fetchApps();
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
        class="px-4 py-2 rounded-lg bg-accent-emerald/15 border border-accent-emerald/30 text-accent-emerald text-sm font-medium hover:bg-accent-emerald/25 transition-all btn-pressable"
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
            v-for="f in ['all', 'third_party', 'system', 'disabled'] as const"
            :key="f"
            class="btn-pressable px-2.5 py-1 rounded-md text-xs font-medium border transition-all"
            :class="[
              appsStore.filter === f
                ? 'bg-accent-emerald/15 border-accent-emerald/30 text-accent-emerald'
                : 'bg-theme-btn border-theme-tertiary text-theme-secondary hover-accent',
            ]"
            @click="
              appsStore.filter = f;
              appsStore.fetchApps();
            "
          >
            {{
              f === 'third_party'
                ? 'Third-Party'
                : f === 'all'
                  ? 'All'
                  : f.charAt(0).toUpperCase() + f.slice(1)
            }}
          </button>
        </div>

        <div class="flex-1" />

        <button
          class="btn-pressable px-3 py-1.5 rounded-lg bg-theme-btn border border-theme-tertiary text-theme-secondary text-xs font-medium hover-accent transition-all"
          :disabled="appsStore.isLoading"
          @click="appsStore.fetchApps()"
        >
          {{ appsStore.isLoading ? 'Loading...' : 'Refresh' }}
        </button>

        <button
          class="btn-pressable px-3 py-1.5 rounded-lg bg-accent-emerald/15 border border-accent-emerald/30 text-accent-emerald text-xs font-medium hover:bg-accent-emerald/25 transition-all"
          :disabled="appsStore.isInstalling"
          @click="appsStore.installApk()"
        >
          {{ appsStore.isInstalling ? 'Installing...' : '+ Install APK' }}
        </button>
      </div>

      <!-- Error -->
      <div
        v-if="appsStore.error"
        class="px-4 py-2 rounded-lg bg-color-error-container border border-color-error text-color-error text-sm"
      >
        {{ appsStore.error }}
        <button
          class="btn-pressable ml-2 underline text-color-error"
          @click="appsStore.fetchApps()"
        >
          Retry
        </button>
      </div>

      <!-- Loading skeleton -->
      <div
        v-if="appsStore.isLoading && appsStore.apps.length === 0"
        class="flex-1 flex items-center justify-center"
      >
        <div class="text-theme-muted text-sm animate-pulse">Loading apps...</div>
      </div>

      <!-- Main content -->
      <div v-else class="flex-1 min-h-0 flex gap-4 overflow-hidden">
        <AppList class="w-[45%] min-w-[280px]" />
        <AppDetail class="flex-1 min-w-0" />
      </div>
    </template>
  </div>
</template>
