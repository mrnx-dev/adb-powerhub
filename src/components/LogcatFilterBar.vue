<script setup lang="ts">
import { useLogcatStore } from '../stores/logcat';
import { Search, Tag } from '@lucide/vue';
import { ref, watch } from 'vue';

const store = useLogcatStore();

const levelOptions: { label: string; value: 'ALL' | 'V' | 'D' | 'I' | 'W' | 'E' | 'F' }[] = [
  { label: 'All', value: 'ALL' },
  { label: 'Verbose', value: 'V' },
  { label: 'Debug', value: 'D' },
  { label: 'Info', value: 'I' },
  { label: 'Warning', value: 'W' },
  { label: 'Error', value: 'E' },
  { label: 'Fatal', value: 'F' },
];

const localTag = ref(store.tagQuery);
const localSearch = ref(store.searchQuery);

let tagDebounce: ReturnType<typeof setTimeout> | null = null;
let searchDebounce: ReturnType<typeof setTimeout> | null = null;

watch(localTag, (val) => {
  if (tagDebounce) clearTimeout(tagDebounce);
  tagDebounce = setTimeout(() => {
    store.tagQuery = val;
  }, 200);
});

watch(localSearch, (val) => {
  if (searchDebounce) clearTimeout(searchDebounce);
  searchDebounce = setTimeout(() => {
    store.searchQuery = val;
  }, 200);
});
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

    <!-- Tag Filter -->
    <div class="flex items-center gap-2 flex-1 min-w-[120px] max-w-[240px]">
      <Tag :size="14" class="text-theme-muted shrink-0" />
      <input
        v-model="localTag"
        type="text"
        placeholder="Filter by tag..."
        class="w-full bg-theme-input border border-theme-tertiary rounded-md px-2 py-1 text-xs text-theme-primary placeholder:text-theme-muted focus:outline-none focus:border-accent-emerald transition-colors"
      />
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
