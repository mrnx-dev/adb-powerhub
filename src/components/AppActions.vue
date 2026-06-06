<script setup lang="ts">
import { useAppsStore } from '../stores/apps';
import { useDeviceStore } from '../stores/device';
import { Power, ToggleLeft, ToggleRight, Eraser, Trash2, Play } from '@lucide/vue';

const appsStore = useAppsStore();
const deviceStore = useDeviceStore();
</script>

<template>
  <div
    class="flex flex-col h-full overflow-hidden rounded-r-lg border border-theme-tertiary bg-theme-surface"
  >
    <!-- Empty: nothing pinned -->
    <div
      v-if="!appsStore.pinnedPackage"
      class="flex-1 flex flex-col items-center justify-center gap-3 text-theme-muted p-6"
    >
      <p class="text-sm text-center">Select an app<br />to manage</p>
    </div>

    <!-- Actions -->
    <div v-else class="flex-1 overflow-y-auto p-4 flex flex-col">
      <!-- Quick Actions -->
      <h4 class="text-xs font-semibold text-theme-muted uppercase tracking-wider mb-2">
        Quick Actions
      </h4>

      <button
        class="btn-pressable flex items-center justify-center gap-2 px-3 py-2.5 rounded-lg bg-accent-light border border-accent-strong text-sm text-accent-emerald hover:bg-accent-default font-medium"
        :disabled="appsStore.isActioning"
        @click="appsStore.forceStopApp(appsStore.pinnedPackage!)"
      >
        <Play :size="14" />
        Open App
      </button>

      <div class="grid grid-cols-2 gap-2 mt-2">
        <button
          class="btn-pressable flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-theme-btn border border-theme-tertiary text-sm text-theme-secondary hover-accent"
          :disabled="appsStore.isActioning"
          @click="appsStore.forceStopApp(appsStore.pinnedPackage!)"
        >
          <Power :size="14" />
          Force Stop
        </button>

        <button
          v-if="appsStore.previewApp && !appsStore.previewApp.is_enabled"
          class="btn-pressable flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-accent-light border border-accent-strong text-sm text-accent-emerald hover:bg-accent-default"
          :disabled="appsStore.isActioning"
          @click="appsStore.enableApp(appsStore.pinnedPackage!)"
        >
          <ToggleRight :size="14" />
          Enable
        </button>

        <button
          v-else
          class="btn-pressable flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-theme-btn border border-theme-tertiary text-sm text-theme-secondary hover-accent"
          :disabled="appsStore.isActioning"
          @click="appsStore.disableApp(appsStore.pinnedPackage!)"
        >
          <ToggleLeft :size="14" />
          Disable
        </button>
      </div>

      <button
        class="btn-pressable flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-theme-btn border border-theme-tertiary text-sm text-theme-secondary hover-accent mt-2 w-full"
        :disabled="appsStore.isActioning"
        @click="appsStore.clearApp(appsStore.pinnedPackage!)"
      >
        <Eraser :size="14" />
        Clear Data
      </button>

      <!-- Dangerous -->
      <h4 class="text-xs font-semibold text-theme-muted uppercase tracking-wider mb-2 mt-6">
        Dangerous
      </h4>

      <button
        class="btn-pressable flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-color-error-container border border-color-error text-sm text-color-error hover:bg-color-error-container w-full"
        :disabled="appsStore.isActioning"
        @click="
          appsStore.uninstallApp(appsStore.pinnedPackage!, appsStore.previewApp?.is_system ?? false)
        "
      >
        <Trash2 :size="14" />
        {{ appsStore.previewApp?.is_system ? 'Disable (System)' : 'Uninstall' }}
      </button>

      <!-- Device info footer -->
      <div
        class="mt-auto pt-3 border-t border-theme-tertiary text-[10px] text-theme-muted space-y-1"
      >
        <div v-if="deviceStore.connected">
          📱 {{ deviceStore.model || 'Connected device' }}
        </div>
      </div>
    </div>
  </div>
</template>
