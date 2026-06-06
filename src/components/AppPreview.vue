<script setup lang="ts">
import { useAppsStore } from '../stores/apps';
import { Package, Pin } from '@lucide/vue';

const appsStore = useAppsStore();

function formatSizeBytes(bytes: number | undefined): string {
  if (bytes == null) return '';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}
</script>

<template>
  <div
    class="flex flex-col h-full overflow-hidden border-t border-b border-theme-tertiary bg-theme-surface"
  >
    <Transition name="preview-switch" mode="out-in">
      <!-- Empty: nothing hovered or pinned -->
      <div
        v-if="!appsStore.previewApp"
        key="empty"
        class="flex-1 flex flex-col items-center justify-center gap-3 text-theme-muted p-6"
      >
        <Package :size="48" class="opacity-25" />
        <p class="text-sm text-center">Hover an app to preview</p>
        <p class="text-xs text-center opacity-60 leading-relaxed">
          <kbd
            class="inline-block px-1.5 py-0.5 rounded text-[10px] bg-theme-btn border border-theme-tertiary font-semibold"
            >↑↓</kbd
          >
          navigate &nbsp;·&nbsp;
          <kbd
            class="inline-block px-1.5 py-0.5 rounded text-[10px] bg-theme-btn border border-theme-tertiary font-semibold"
            >Enter</kbd
          >
          select &nbsp;·&nbsp;
          <kbd
            class="inline-block px-1.5 py-0.5 rounded text-[10px] bg-theme-btn border border-theme-tertiary font-semibold"
            >Space</kbd
          >
          peek
        </p>
      </div>

      <!-- Preview content -->
      <div v-else key="content" class="flex-1 overflow-y-auto p-4">
        <!-- Header with icon -->
        <div class="flex items-start justify-between gap-3 mb-4">
          <div class="min-w-0">
            <h3
              class="text-lg font-semibold text-theme-primary break-words flex items-center gap-2"
            >
              <Pin v-if="appsStore.pinnedPackage" :size="12" class="text-accent-emerald shrink-0" />
              {{ appsStore.previewApp.label }}
            </h3>
            <div class="flex items-center gap-2 mt-1">
              <span class="text-xs text-theme-muted truncate max-w-[280px]">
                {{ appsStore.previewApp.package_name }}
              </span>
            </div>
          </div>

          <div class="w-16 h-16 shrink-0">
            <img
              v-if="appsStore.icons[appsStore.previewApp.package_name]"
              :src="appsStore.icons[appsStore.previewApp.package_name]"
              class="w-16 h-16 rounded-2xl object-cover icon-fade-in"
              :alt="appsStore.previewApp.label"
            />
            <div
              v-else
              class="w-16 h-16 rounded-2xl flex items-center justify-center text-xl font-bold"
              :class="[
                appsStore.previewApp.is_system
                  ? 'bg-color-warning-container text-color-warning'
                  : 'bg-accent-light text-accent-emerald',
              ]"
            >
              {{ appsStore.previewApp.label.charAt(0) }}
            </div>
          </div>
        </div>

        <!-- Badges -->
        <div class="flex flex-wrap gap-2 mb-4">
          <span
            v-if="appsStore.previewApp.is_system"
            class="px-2 py-0.5 rounded-md text-xs font-semibold bg-color-warning-container text-color-warning"
          >
            System
          </span>
          <span
            v-if="!appsStore.previewApp.is_enabled"
            class="px-2 py-0.5 rounded-md text-xs font-semibold bg-color-error-container text-color-error"
          >
            Disabled
          </span>
          <span
            v-if="appsStore.previewApp.is_updated_system"
            class="px-2 py-0.5 rounded-md text-xs font-semibold bg-color-info-container text-color-info"
          >
            Updated System App
          </span>
        </div>

        <!-- Info -->
        <div class="space-y-2 text-sm">
          <div v-if="appsStore.previewApp.version_name" class="flex justify-between">
            <span class="text-theme-muted">Version</span>
            <span class="text-theme-primary">{{ appsStore.previewApp.version_name }}</span>
          </div>
          <div v-if="appsStore.previewApp.version_code" class="flex justify-between">
            <span class="text-theme-muted">Version Code</span>
            <span class="text-theme-primary">{{ appsStore.previewApp.version_code }}</span>
          </div>
          <div v-if="appsStore.previewApp.apk_size" class="flex justify-between">
            <span class="text-theme-muted">APK Size</span>
            <span class="text-theme-primary">{{
              formatSizeBytes(appsStore.previewApp.apk_size)
            }}</span>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>
