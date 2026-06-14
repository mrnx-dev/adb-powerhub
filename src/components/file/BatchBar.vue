<script setup lang="ts">
import { computed } from 'vue';
import { useFileExplorerStore } from '../../stores/fileExplorer';
import { useDeviceStore } from '../../stores/device';
import { Download, Trash2 } from '@lucide/vue';

const store = useFileExplorerStore();
const deviceStore = useDeviceStore();

const count = computed(() => store.selectedPaths.size);
const paths = computed(() => Array.from(store.selectedPaths));

function onPull() {
  store.pull(paths.value);
}

function onDelete() {
  store.deletePaths(paths.value);
}
</script>

<template>
  <Transition name="batch-bar">
    <div v-if="store.showBatchBar" class="batch-bar" role="toolbar" aria-label="Batch actions">
      <span class="batch-count">{{ count }} selected</span>
      <div class="batch-separator" />
      <button
        class="batch-button"
        :disabled="!deviceStore.connected || store.isPulling"
        @click="onPull"
      >
        <Download :size="16" />
        <span>Pull</span>
      </button>
      <button
        class="batch-button danger"
        :disabled="!deviceStore.connected || store.isDeleting"
        @click="onDelete"
      >
        <Trash2 :size="16" />
        <span>Delete</span>
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.batch-bar {
  @apply fixed bottom-6 left-1/2 -translate-x-1/2 flex items-center gap-3 px-5 py-3 rounded-xl border border-theme-tertiary bg-theme-card shadow-xl z-40;
}

.batch-bar-enter-active,
.batch-bar-leave-active {
  transition:
    transform 250ms cubic-bezier(0, 0, 0.2, 1),
    opacity 250ms ease;
}

.batch-bar-enter-from,
.batch-bar-leave-to {
  transform: translateX(-50%) translateY(100px);
  opacity: 0;
}

.batch-count {
  @apply text-sm font-semibold text-accent-emerald;
}

.batch-separator {
  @apply w-px h-6 bg-theme-tertiary;
}

.batch-button {
  @apply flex items-center gap-1.5 px-3 py-2 rounded-lg text-sm text-theme-primary transition-colors disabled:opacity-40;
}

.batch-button:not(:disabled):hover {
  @apply bg-theme-hover;
}

.batch-button.danger:not(:disabled):hover {
  @apply bg-color-error-container text-color-error;
}

@media (prefers-reduced-motion: reduce) {
  .batch-bar-enter-active,
  .batch-bar-leave-active {
    transition: none;
  }
}
</style>
