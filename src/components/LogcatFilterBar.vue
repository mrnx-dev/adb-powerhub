<script setup lang="ts">
import { useLogcatStore } from '../stores/logcat';
import { useDeviceStore } from '../stores/device';
import { Search, Tag, Smartphone, X } from '@lucide/vue';
import { ref, watch } from 'vue';

const store = useLogcatStore();
const deviceStore = useDeviceStore();

const levelOptions: { label: string; value: 'ALL' | 'V' | 'D' | 'I' | 'W' | 'E' | 'F' }[] = [
  { label: 'All', value: 'ALL' },
  { label: 'Verbose', value: 'V' },
  { label: 'Debug', value: 'D' },
  { label: 'Info', value: 'I' },
  { label: 'Warning', value: 'W' },
  { label: 'Error', value: 'E' },
  { label: 'Fatal', value: 'F' },
];

const tagInput = ref('');
const localSearch = ref(store.searchQuery);

let searchDebounce: ReturnType<typeof setTimeout> | null = null;

watch(localSearch, (val) => {
  if (searchDebounce) clearTimeout(searchDebounce);
  searchDebounce = setTimeout(() => {
    store.searchQuery = val;
  }, 200);
});

function onTagKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' || e.key === ',') {
    e.preventDefault();
    addTagFromInput();
  }
}

function addTagFromInput() {
  if (!tagInput.value.trim()) return;
  store.addTagFilter(tagInput.value);
  tagInput.value = '';
}

function onActiveAppToggle(e: Event) {
  const checked = (e.target as HTMLInputElement).checked;
  store.setActiveAppOnly(checked);
}

function chipClasses(mode: 'include' | 'exclude') {
  return mode === 'include'
    ? 'bg-accent-emerald/10 text-accent-emerald border-accent-emerald/25'
    : 'bg-red-500/10 text-red-400 border-red-500/25';
}
</script>

<template>
  <div class="flex items-center gap-3 shrink-0 flex-wrap">
    <!-- Level Filter -->
    <div class="flex items-center gap-2">
      <label class="text-xs text-theme-muted">Level</label>
      <select
        v-model="store.filterLevel"
        class="bg-theme-input border border-theme-tertiary rounded-md px-2 py-1 text-xs text-theme-primary focus:outline-none focus:border-accent-emerald transition-colors"
      >
        <option v-for="opt in levelOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
    </div>

    <!-- Active App Filter -->
    <div class="flex items-center gap-2">
      <label
        class="flex items-center gap-1.5 cursor-pointer"
        :class="{
          'opacity-50 cursor-not-allowed': !deviceStore.connected || !store.streaming,
        }"
      >
        <input
          :checked="store.activeAppOnly"
          type="checkbox"
          class="accent-accent-emerald"
          :disabled="!deviceStore.connected || !store.streaming"
          @change="onActiveAppToggle"
        />
        <Smartphone :size="14" class="text-theme-muted shrink-0" />
        <span class="text-xs text-theme-primary whitespace-nowrap">Current App Only</span>
      </label>
      <span
        v-if="store.activeAppOnly && store.activeAppPackage"
        class="text-[11px] px-2 py-0.5 rounded bg-accent-emerald/10 text-accent-emerald border border-accent-emerald/20 max-w-[140px] truncate"
        :title="store.activeAppPackage"
      >
        {{ store.activeAppPackage }}
      </span>
    </div>

    <!-- Tag Filter Chips -->
    <div class="flex items-center gap-2 flex-1 min-w-[200px]">
      <Tag :size="14" class="text-theme-muted shrink-0" />
      <div
        class="flex items-center gap-1.5 flex-wrap flex-1 bg-theme-input border border-theme-tertiary rounded-md px-2 py-1 min-h-[28px]"
      >
        <span
          v-for="(filter, i) in store.tagFilters"
          :key="filter.value + filter.mode"
          class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] font-medium border cursor-pointer select-none transition-colors"
          :class="chipClasses(filter.mode)"
          @click="store.toggleTagMode(i)"
        >
          <span>{{ filter.mode === 'include' ? '+' : '−' }}</span>
          <span class="max-w-[100px] truncate">{{ filter.value }}</span>
          <button class="hover:opacity-80" @click.stop="store.removeTagFilter(i)">
            <X :size="10" />
          </button>
        </span>

        <input
          v-model="tagInput"
          type="text"
          placeholder="Type tag & press Enter..."
          class="bg-transparent text-xs text-theme-primary placeholder:text-theme-muted focus:outline-none min-w-[120px] flex-1"
          @keydown="onTagKeydown"
          @blur="addTagFromInput"
        />
      </div>

      <button
        v-if="store.tagFilters.length > 0"
        class="text-[11px] text-theme-muted hover:text-red-400 transition-colors"
        @click="store.clearTagFilters"
      >
        Clear
      </button>
    </div>

    <!-- Search -->
    <div class="flex items-center gap-2 flex-1 min-w-[120px]">
      <Search :size="14" class="text-theme-muted shrink-0" />
      <input
        v-model="localSearch"
        type="text"
        placeholder="Search logs..."
        class="w-full bg-theme-input border border-theme-tertiary rounded-md px-2 py-1 text-xs text-theme-primary placeholder:text-theme-muted focus:outline-none focus:border-accent-emerald transition-colors"
      />
    </div>
  </div>
</template>
