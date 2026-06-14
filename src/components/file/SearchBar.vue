<script setup lang="ts">
import { ref, watch } from 'vue';
import { useFileExplorerStore } from '../../stores/fileExplorer';
import { Search, X } from '@lucide/vue';

const store = useFileExplorerStore();
const inputRef = ref<HTMLInputElement | null>(null);
const localQuery = ref(store.searchQuery);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(localQuery, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    store.setSearch(val);
  }, 200);
});

function clearSearch() {
  localQuery.value = '';
  store.setSearch('');
  inputRef.value?.focus();
}

function onKeydown(e: Event) {
  const ke = e as KeyboardEvent;
  if ((ke.ctrlKey || ke.metaKey) && ke.key === 'f') {
    e.preventDefault();
    inputRef.value?.focus();
  }
}

useEventListener(document, 'keydown', onKeydown);

function useEventListener(target: EventTarget, event: string, handler: (e: Event) => void) {
  import('vue').then((vue) => {
    vue.onMounted(() => target.addEventListener(event, handler));
    vue.onBeforeUnmount(() => target.removeEventListener(event, handler));
  });
}
</script>

<template>
  <div class="flex items-center gap-2">
    <div class="relative flex items-center">
      <Search :size="14" class="absolute left-2.5 text-theme-muted pointer-events-none" />
      <input
        ref="inputRef"
        v-model="localQuery"
        type="text"
        placeholder="Search files..."
        class="pl-8 pr-7 py-1.5 rounded-lg bg-theme-btn border border-theme-tertiary text-sm text-theme-primary placeholder:text-theme-muted focus:outline-none focus:border-accent-focus w-40 sm:w-56"
      />
      <button
        v-if="localQuery"
        class="absolute right-1.5 p-1 rounded hover:bg-theme-hover text-theme-muted"
        @click="clearSearch"
      >
        <X :size="12" />
      </button>
    </div>
    <span v-if="localQuery" class="text-[11px] text-theme-muted whitespace-nowrap">
      {{ store.filteredEntries.length }}/{{ store.totalCount }}
    </span>
  </div>
</template>
