<script setup lang="ts">
import { useAppsStore } from '../stores/apps';
import { useToastStore } from '../stores/toast';
import { Package, Trash2, Eraser, Power, ToggleLeft, ToggleRight, Copy } from '@lucide/vue';

const appsStore = useAppsStore();
const toast = useToastStore();

function copyPackageName() {
  if (!appsStore.appDetail) return;
  navigator.clipboard.writeText(appsStore.appDetail.package_name);
  toast.show('Copied to clipboard', 'success');
}
</script>

<template>
  <div
    class="flex flex-col h-full overflow-hidden rounded-lg border border-theme-tertiary bg-theme-surface"
  >
    <!-- Empty state -->
    <div
      v-if="!appsStore.appDetail"
      class="flex-1 flex flex-col items-center justify-center gap-3 text-theme-muted"
    >
      <Package :size="48" class="opacity-30" />
      <p class="text-sm">Select an app to view details</p>
    </div>

    <!-- Detail -->
    <template v-else>
      <div class="flex-1 overflow-y-auto p-4">
        <!-- Header -->
        <div class="flex items-start justify-between gap-3 mb-4">
          <div class="min-w-0">
            <h3 class="text-lg font-semibold text-theme-primary break-words">
              {{ appsStore.appDetail.label }}
            </h3>
            <div class="flex items-center gap-2 mt-1">
              <span class="text-xs text-theme-muted truncate max-w-[280px]">
                {{ appsStore.appDetail.package_name }}
              </span>
              <button
                class="btn-pressable p-0.5 rounded hover:bg-theme-hover text-theme-muted hover:text-theme-primary transition-colors"
                title="Copy package name"
                @click="copyPackageName()"
              >
                <Copy :size="12" />
              </button>
            </div>
          </div>

          <div class="relative w-16 h-16 shrink-0">
            <img
              v-if="appsStore.icons[appsStore.appDetail.package_name]"
              :src="appsStore.icons[appsStore.appDetail.package_name]"
              class="absolute inset-0 w-16 h-16 rounded-2xl object-cover icon-fade-in"
              :alt="appsStore.appDetail.label"
            />
            <div
              class="w-16 h-16 rounded-2xl flex items-center justify-center text-xl font-bold"
              :class="[
                appsStore.appDetail.is_system
                  ? 'bg-color-warning-container text-color-warning'
                  : 'bg-accent-light text-accent-emerald',
              ]"
            >
              {{ appsStore.appDetail.label.charAt(0) }}
            </div>
          </div>
        </div>

        <!-- Badges -->
        <div class="flex flex-wrap gap-2 mb-4">
          <span
            v-if="appsStore.appDetail.is_system"
            class="px-2 py-0.5 rounded-md text-xs font-semibold bg-color-warning-container text-color-warning"
          >
            System
          </span>
          <span
            v-if="!appsStore.appDetail.is_enabled"
            class="px-2 py-0.5 rounded-md text-xs font-semibold bg-color-error-container text-color-error"
          >
            Disabled
          </span>
          <span
            v-if="appsStore.appDetail.is_updated_system"
            class="px-2 py-0.5 rounded-md text-xs font-semibold bg-color-info-container text-color-info"
          >
            Updated System App
          </span>
        </div>

        <!-- Info -->
        <div class="space-y-2 text-sm">
          <div v-if="appsStore.appDetail.version_name" class="flex justify-between">
            <span class="text-theme-muted">Version</span>
            <span class="text-theme-primary">{{ appsStore.appDetail.version_name }}</span>
          </div>
          <div v-if="appsStore.appDetail.version_code" class="flex justify-between">
            <span class="text-theme-muted">Version Code</span>
            <span class="text-theme-primary">{{ appsStore.appDetail.version_code }}</span>
          </div>
        </div>

        <!-- Actions -->
        <div class="mt-6 space-y-2">
          <h4 class="text-xs font-semibold text-theme-muted uppercase tracking-wider mb-2">
            Actions
          </h4>

          <div class="grid grid-cols-2 gap-2">
            <button
              class="btn-pressable flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-theme-btn border border-theme-tertiary text-sm text-theme-secondary hover-accent transition-all"
              :disabled="appsStore.isActioning"
              @click="appsStore.forceStopApp(appsStore.appDetail!.package_name)"
            >
              <Power :size="14" />
              Force Stop
            </button>

            <button
              v-if="!appsStore.appDetail.is_enabled"
              class="btn-pressable flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-accent-light border border-accent-strong text-sm text-accent-emerald hover:bg-accent-default transition-all"
              :disabled="appsStore.isActioning"
              @click="appsStore.enableApp(appsStore.appDetail!.package_name)"
            >
              <ToggleRight :size="14" />
              Enable
            </button>

            <button
              v-else
              class="btn-pressable flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-theme-btn border border-theme-tertiary text-sm text-theme-secondary hover-accent transition-all"
              :disabled="appsStore.isActioning"
              @click="appsStore.disableApp(appsStore.appDetail!.package_name)"
            >
              <ToggleLeft :size="14" />
              Disable
            </button>

            <button
              class="btn-pressable flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-theme-btn border border-theme-tertiary text-sm text-theme-secondary hover-accent transition-all"
              :disabled="appsStore.isActioning"
              @click="appsStore.clearApp(appsStore.appDetail!.package_name)"
            >
              <Eraser :size="14" />
              Clear Data
            </button>

            <button
              class="btn-pressable flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-color-error-container border border-color-error text-sm text-color-error hover:bg-color-error-container transition-all"
              :disabled="appsStore.isActioning"
              @click="
                appsStore.uninstallApp(
                  appsStore.appDetail!.package_name,
                  appsStore.appDetail!.is_system
                )
              "
            >
              <Trash2 :size="14" />
              {{ appsStore.appDetail.is_system ? 'Disable (Sys)' : 'Uninstall' }}
            </button>
          </div>
        </div>

        <!-- Technical Info -->
        <details class="mt-6">
          <summary
            class="text-xs font-semibold text-theme-muted uppercase tracking-wider cursor-pointer hover:text-theme-primary transition-colors"
          >
            Technical Info
          </summary>
          <div class="mt-2 space-y-1.5 text-xs">
            <div v-if="appsStore.appDetail.code_path" class="flex justify-between">
              <span class="text-theme-muted">Code Path</span>
              <span class="text-theme-primary break-all text-right max-w-[60%]">{{
                appsStore.appDetail.code_path
              }}</span>
            </div>
            <div v-if="appsStore.appDetail.data_dir" class="flex justify-between">
              <span class="text-theme-muted">Data Dir</span>
              <span class="text-theme-primary break-all text-right max-w-[60%]">{{
                appsStore.appDetail.data_dir
              }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-theme-muted">Package</span>
              <span class="text-theme-primary break-all text-right max-w-[60%]">{{
                appsStore.appDetail.package_name
              }}</span>
            </div>
          </div>
        </details>
      </div>
    </template>
  </div>
</template>
