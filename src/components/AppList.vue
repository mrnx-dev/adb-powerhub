<script setup lang="ts">
import { ref, watch, onUnmounted, nextTick } from 'vue';
import { useAppsStore } from '../stores/apps';
import { ArrowUpDown } from '@lucide/vue';

const appsStore = useAppsStore();
const searchInput = ref('');
const searchInputRef = ref<HTMLInputElement | null>(null);
const listScrollRef = ref<HTMLElement | null>(null);
const keyboardIndex = ref(-1);
let searchTimeout: ReturnType<typeof setTimeout> | null = null;

function formatSizeBytes(bytes: number | undefined): string {
  if (bytes == null) return '';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

const hasBadges = (app: { is_system: boolean; is_enabled: boolean; is_updated_system: boolean }) =>
  app.is_system || !app.is_enabled || app.is_updated_system;

watch(searchInput, (val) => {
  if (searchTimeout) clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    appsStore.searchQuery = val;
    keyboardIndex.value = -1;
  }, 300);
});

onUnmounted(() => {
  if (searchTimeout) clearTimeout(searchTimeout);
});

// Reset keyboard index when list changes (filter, sort, new data)
watch(
  () => appsStore.filteredApps.length,
  () => {
    keyboardIndex.value = -1;
  }
);

function scrollToItem(index: number) {
  nextTick(() => {
    const items = listScrollRef.value?.querySelectorAll('button');
    items?.[index]?.scrollIntoView({ block: 'nearest' });
  });
}

function handleKeydown(e: KeyboardEvent) {
  const filtered = appsStore.filteredApps;
  if (filtered.length === 0) return;

  const isSearchInput = e.target instanceof HTMLInputElement;

  switch (e.key) {
    case 'ArrowDown':
      if (!isSearchInput) {
        e.preventDefault();
        keyboardIndex.value = Math.min(keyboardIndex.value + 1, filtered.length - 1);
        const app = filtered[keyboardIndex.value];
        appsStore.hoverApp(app.package_name);
        scrollToItem(keyboardIndex.value);
      }
      break;
    case 'ArrowUp':
      if (!isSearchInput) {
        e.preventDefault();
        keyboardIndex.value = Math.max(keyboardIndex.value - 1, 0);
        const app = filtered[keyboardIndex.value];
        appsStore.hoverApp(app.package_name);
        scrollToItem(keyboardIndex.value);
      }
      break;
    case 'Enter':
      if (!isSearchInput) {
        e.preventDefault();
        if (keyboardIndex.value >= 0) {
          const app = filtered[keyboardIndex.value];
          appsStore.selectApp(app.package_name);
        }
      }
      break;
    case 'Escape':
      e.preventDefault();
      if (isSearchInput) {
        searchInput.value = '';
        appsStore.searchQuery = '';
      }
      appsStore.clearPin();
      appsStore.unhoverApp();
      keyboardIndex.value = -1;
      break;
    case '/':
      if (!isSearchInput) {
        e.preventDefault();
        searchInputRef.value?.focus();
      }
      break;
  }
}
</script>

<template>
  <div
    class="flex flex-col h-full overflow-hidden rounded-lg border border-theme-tertiary bg-theme-surface outline-none"
    tabindex="0"
    @keydown="handleKeydown"
  >
    <!-- Search + sort header -->
    <div class="px-3 py-2 border-b border-theme-tertiary flex items-center gap-2">
      <input
        ref="searchInputRef"
        v-model="searchInput"
        type="text"
        placeholder="Search apps..."
        class="flex-1 min-w-0 px-3 py-1.5 rounded-md bg-theme-btn border border-theme-tertiary text-sm text-theme-primary placeholder:text-theme-muted focus:outline-none focus:border-accent-focus"
      />
      <button
        class="btn-pressable shrink-0 flex items-center gap-1 px-2 py-1.5 rounded-md text-[11px] text-theme-muted hover:text-theme-primary bg-theme-btn border border-theme-tertiary"
        :aria-label="`Sort by ${appsStore.sortBy === 'name' ? 'name' : 'size'}`"
        @click="appsStore.sortBy = appsStore.sortBy === 'name' ? 'size' : 'name'"
      >
        <ArrowUpDown :size="12" />
        <span class="hidden sm:inline">{{ appsStore.sortBy === 'name' ? 'Name' : 'Size' }}</span>
      </button>
    </div>

    <!-- App list -->
    <div ref="listScrollRef" class="app-list-scroll flex-1 overflow-y-auto">
      <div
        v-if="appsStore.filteredApps.length === 0 && !appsStore.isLoading"
        class="flex flex-col items-center justify-center py-12 text-theme-muted text-sm"
      >
        No apps match your search
      </div>

      <TransitionGroup name="card-stagger" tag="div">
        <button
          v-for="(app, index) in appsStore.filteredApps"
          :key="app.package_name"
          :style="[
            { '--stagger-index': index },
            appsStore.pinnedPackage === app.package_name
              ? { boxShadow: 'inset 4px 0 0 var(--color-accent-emerald)' }
              : {},
          ]"
          class="btn-pressable w-full flex items-center gap-3 px-3 py-2.5 text-left hover-subtle group"
          :class="[
            appsStore.pinnedPackage === app.package_name ? 'bg-accent-light' : '',
            keyboardIndex === index ? 'keyboard-focus' : '',
          ]"
          :aria-label="`${app.label} - ${app.package_name}`"
          @click="appsStore.selectApp(app.package_name)"
          @mouseenter="appsStore.hoverApp(app.package_name)"
          @mouseleave="appsStore.unhoverApp()"
        >
          <!-- Icon: real when available, fallback otherwise -->
          <div class="w-12 h-12 shrink-0">
            <img
              v-if="appsStore.icons[app.package_name]"
              :src="appsStore.icons[app.package_name]"
              class="w-12 h-12 rounded-xl object-cover app-list-icon-hover icon-fade-in"
              :alt="app.label"
            />
            <div
              v-else
              class="w-12 h-12 rounded-xl flex items-center justify-center text-xs font-bold"
              :class="[
                app.is_system
                  ? 'bg-color-warning-container text-color-warning'
                  : 'bg-accent-light text-accent-emerald',
              ]"
            >
              {{ app.label.charAt(0) }}
            </div>
          </div>

          <div class="min-w-0 flex-1">
            <div class="text-sm font-medium text-theme-primary truncate">
              {{ app.label }}
            </div>
            <div class="text-xs text-theme-muted truncate">
              {{ app.package_name }}
            </div>
          </div>

          <!-- Meta: size + badges -->
          <div class="app-meta flex flex-col items-end gap-0.5 shrink-0">
            <span class="text-[10px] text-theme-muted leading-none">
              {{ formatSizeBytes(app.apk_size) }}
            </span>
            <div v-if="hasBadges(app)" class="flex flex-wrap gap-1 justify-end">
              <span
                v-if="app.is_system"
                class="px-1.5 py-0.5 rounded text-[10px] font-semibold bg-color-warning-container text-color-warning"
              >
                SYS
              </span>
              <span
                v-if="!app.is_enabled"
                class="px-1.5 py-0.5 rounded text-[10px] font-semibold bg-color-error-container text-color-error"
              >
                DIS
              </span>
              <span
                v-if="app.is_updated_system"
                class="px-1.5 py-0.5 rounded text-[10px] font-semibold bg-color-info-container text-color-info"
              >
                UPD
              </span>
            </div>
          </div>
        </button>
      </TransitionGroup>
    </div>
  </div>
</template>

<style scoped>
@media (hover: hover) and (pointer: fine) {
  .app-list-icon-hover {
    transition: transform 200ms ease-out;
  }
  .btn-pressable:hover .app-list-icon-hover {
    transform: scale(1.05);
  }
}
</style>
