<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useFileExplorerStore } from '../../stores/fileExplorer';
import { useDeviceStore } from '../../stores/device';
import type { FileNode } from '../../stores/fileExplorer';
import { FolderOpen, Download, Upload, Copy, Pencil, Trash2 } from '@lucide/vue';

const props = defineProps<{
  x: number;
  y: number;
  node: FileNode | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const store = useFileExplorerStore();
const deviceStore = useDeviceStore();
const menuRef = ref<HTMLElement | null>(null);

function close() {
  emit('close');
}

function onPull() {
  if (!props.node) return;
  close();
  store.pull([props.node.path]);
}

function onPush() {
  close();
  store.push();
}

function onOpen() {
  if (!props.node) return;
  close();
  if (props.node.entryType === 'folder') {
    store.navigateTo(props.node.path);
  } else {
    store.pull([props.node.path]);
  }
}

function onCopyPath() {
  if (!props.node) return;
  navigator.clipboard.writeText(props.node.path);
  close();
}

function onRename() {
  if (!props.node) return;
  close();
  store.startRename(props.node.path);
}

function onDelete() {
  if (!props.node) return;
  close();
  store.deletePaths([props.node.path]);
}

function onClickOutside(e: MouseEvent) {
  if (!menuRef.value?.contains(e.target as Node)) {
    close();
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') close();
}

onMounted(() => {
  // Adjust position to keep inside viewport
  const rect = menuRef.value?.getBoundingClientRect();
  if (rect) {
    // TODO: shift if overflowing; left as-is for MVP
  }
  document.addEventListener('click', onClickOutside);
  document.addEventListener('keydown', onKeydown);
});

onUnmounted(() => {
  document.removeEventListener('click', onClickOutside);
  document.removeEventListener('keydown', onKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div ref="menuRef" class="context-menu" :style="{ left: `${x}px`, top: `${y}px` }" role="menu">
      <button class="context-menu-item" role="menuitem" @click="onOpen">
        <FolderOpen :size="16" />
        <span>{{ node?.entryType === 'folder' ? 'Open' : 'Pull to PC' }}</span>
        <span class="context-menu-shortcut">Enter</span>
      </button>

      <button class="context-menu-item" role="menuitem" @click="onPull">
        <Download :size="16" />
        <span>Pull to PC</span>
      </button>

      <button
        class="context-menu-item"
        role="menuitem"
        :disabled="!deviceStore.connected"
        @click="onPush"
      >
        <Upload :size="16" />
        <span>Push from PC</span>
      </button>

      <div class="context-menu-separator" />

      <button class="context-menu-item" role="menuitem" @click="onCopyPath">
        <Copy :size="16" />
        <span>Copy Path</span>
        <span class="context-menu-shortcut">Ctrl+C</span>
      </button>

      <button class="context-menu-item" role="menuitem" @click="onRename">
        <Pencil :size="16" />
        <span>Rename</span>
        <span class="context-menu-shortcut">F2</span>
      </button>

      <div class="context-menu-separator" />

      <button class="context-menu-item danger" role="menuitem" @click="onDelete">
        <Trash2 :size="16" />
        <span>Delete</span>
        <span class="context-menu-shortcut">Del</span>
      </button>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu {
  @apply fixed z-[200] min-w-[200px] p-1.5 rounded-lg border border-theme-tertiary bg-theme-card shadow-lg;
  animation: menuIn 120ms ease-out;
}

@keyframes menuIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.context-menu-item {
  @apply w-full flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-theme-primary transition-colors disabled:opacity-40;
}

.context-menu-item:not(:disabled):hover {
  @apply bg-theme-hover;
}

.context-menu-item.danger:not(:disabled):hover {
  @apply bg-color-error-container text-color-error;
}

.context-menu-separator {
  @apply h-px bg-theme-tertiary my-1;
}

.context-menu-shortcut {
  @apply ml-auto text-[11px] text-theme-muted;
}

@media (prefers-reduced-motion: reduce) {
  .context-menu {
    animation: none;
  }
}
</style>
