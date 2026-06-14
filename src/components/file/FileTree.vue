<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { useFileExplorerStore } from '../../stores/fileExplorer';
import { useFileKeyboard } from '../../composables/useFileKeyboard';
import { useFileSelection } from '../../composables/useFileSelection';
import FileTreeRow from './FileTreeRow.vue';
import ContextMenu from './ContextMenu.vue';
import type { FileNode } from '../../stores/fileExplorer';

const store = useFileExplorerStore();
const containerRef = ref<HTMLElement | null>(null);
const { handleKeydown } = useFileKeyboard(containerRef);
useFileSelection();
const contextMenuOpen = ref(false);
const contextMenuPos = ref({ x: 0, y: 0 });
const contextNode = ref<FileNode | null>(null);

function openContextMenu(e: MouseEvent, node: FileNode) {
  contextMenuPos.value = { x: e.clientX, y: e.clientY };
  contextNode.value = node;
  contextMenuOpen.value = true;
}

function closeContextMenu() {
  contextMenuOpen.value = false;
  contextNode.value = null;
}

onMounted(() => {
  containerRef.value?.focus();
});

onUnmounted(() => {
  closeContextMenu();
});

watch(
  () => store.focusedPath,
  (path) => {
    if (!path) return;
    const el = containerRef.value?.querySelector(`[data-path="${CSS.escape(path)}"]`);
    el?.scrollIntoView({ block: 'nearest' });
  }
);
</script>

<template>
  <div ref="containerRef" class="outline-none" tabindex="0" @keydown="handleKeydown">
    <FileTreeRow
      v-for="(node, index) in store.filteredEntries"
      :key="node.path"
      :node="node"
      :index="index"
      :data-path="node.path"
      :style="{ '--stagger-index': index }"
      class="stagger-in"
      @contextmenu="openContextMenu"
    />

    <ContextMenu
      v-if="contextMenuOpen"
      :x="contextMenuPos.x"
      :y="contextMenuPos.y"
      :node="contextNode"
      @close="closeContextMenu"
    />
  </div>
</template>

<style scoped>
.stagger-in {
  animation: fadeInRow 180ms ease-out backwards;
  animation-delay: calc(min(var(--stagger-index), 20) * 20ms);
}

@keyframes fadeInRow {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .stagger-in {
    animation: none;
  }
}
</style>
