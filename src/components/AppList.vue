<script setup lang="ts">
import { ref, watch } from 'vue';
import { useAppsStore } from '../stores/apps';

const appsStore = useAppsStore();
const searchInput = ref('');
let searchTimeout: ReturnType<typeof setTimeout> | null = null;

watch(searchInput, (val) => {
  if (searchTimeout) clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    appsStore.searchQuery = val;
  }, 300);
});
</script>

<template>
  <div
    class="flex flex-col h-full overflow-hidden rounded-lg border border-theme-tertiary bg-theme-surface"
  >
    <!-- Search -->
    <div class="px-3 py-2 border-b border-theme-tertiary">
      <input
        v-model="searchInput"
        type="text"
        placeholder="Search apps..."
        class="w-full px-3 py-1.5 rounded-md bg-theme-btn border border-theme-tertiary text-sm text-theme-primary placeholder:text-theme-muted focus:outline-none focus:border-accent-focus"
      />
    </div>

    <!-- App list -->
    <div class="flex-1 overflow-y-auto">
      <div
        v-if="appsStore.filteredApps.length === 0 && !appsStore.isLoading"
        class="flex flex-col items-center justify-center py-12 text-theme-muted text-sm"
      >
        No apps match your search
      </div>

      <TransitionGroup name="card-stagger" tag="div">
        <button
          v-for="app in appsStore.filteredApps"
          :key="app.package_name"
          class="btn-pressable w-full flex items-center gap-3 px-3 py-2.5 border-b border-theme-tertiary/50 text-left transition-all hover:bg-theme-hover"
          :class="[
            appsStore.selectedPackage === app.package_name
              ? 'bg-accent-light border-l-4 border-l-accent-emerald'
              : '',
          ]"
          @click="appsStore.selectApp(app.package_name)"
        >
          <!-- Icon: first letter -->
          <div
            class="w-9 h-9 shrink-0 rounded-lg flex items-center justify-center text-xs font-bold"
            :class="[
              app.is_system ? 'bg-color-warning-container text-color-warning' : 'bg-accent-light text-accent-emerald',
            ]"
          >
            {{ app.label.charAt(0) }}
          </div>

          <div class="min-w-0 flex-1">
            <div class="text-sm font-medium text-theme-primary truncate">
              {{ app.label }}
            </div>
            <div class="text-xs text-theme-muted truncate">
              {{ app.package_name }}
            </div>
          </div>

          <!-- Badges -->
          <div class="flex flex-wrap gap-1 shrink-0">
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
        </button>
      </TransitionGroup>
    </div>
  </div>
</template>
