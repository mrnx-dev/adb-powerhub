<script setup lang="ts">
import { computed } from 'vue';
import { useFileExplorerStore } from '../../stores/fileExplorer';
import { ChevronRight, Folder } from '@lucide/vue';

const store = useFileExplorerStore();

const segments = computed(() => {
  const p = store.currentPath.replace(/^\/+/, '').replace(/\/+$/, '');
  if (!p) return [];
  return p.split('/');
});

function segmentPath(index: number) {
  return '/' + segments.value.slice(0, index + 1).join('/');
}

function rootLabel() {
  const root = store.allowedRoot.replace(/^\/+/, '').split('/')[0];
  return root || 'device';
}
</script>

<template>
  <div class="flex items-center gap-1">
    <Folder :size="16" class="text-accent-emerald shrink-0" />
    <button
      class="breadcrumb-segment"
      :class="{ current: segments.length === 0 }"
      @click="store.navigateTo(store.allowedRoot)"
    >
      {{ rootLabel() }}
    </button>
    <template v-for="(segment, index) in segments" :key="index">
      <ChevronRight :size="14" class="text-theme-muted shrink-0" />
      <button
        class="breadcrumb-segment"
        :class="{ current: index === segments.length - 1 }"
        @click="store.navigateTo(segmentPath(index))"
      >
        {{ segment }}
      </button>
    </template>
  </div>
</template>

<style scoped>
.breadcrumb-segment {
  @apply px-2 py-1 rounded-md text-xs text-theme-secondary transition-colors;
}

.breadcrumb-segment:not(.current):hover {
  @apply bg-theme-hover text-theme-primary;
}

.breadcrumb-segment.current {
  @apply font-semibold text-theme-primary cursor-default;
}
</style>
