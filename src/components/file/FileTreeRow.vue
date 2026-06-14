<script setup lang="ts">
import { computed } from 'vue';
import { useFileExplorerStore } from '../../stores/fileExplorer';
import type { FileNode } from '../../stores/fileExplorer';
import {
  Folder,
  FileImage,
  FileCode,
  FileText,
  FileArchive,
  FileVideo,
  Music,
  File,
  ChevronRight,
} from '@lucide/vue';

const props = defineProps<{
  node: FileNode;
  index: number;
}>();

const store = useFileExplorerStore();

const isSelected = computed(() => store.selectedPaths.has(props.node.path));
const isFocused = computed(() => store.focusedPath === props.node.path);
const isFolder = computed(() => props.node.entryType === 'folder');

const iconComponent = computed(() => {
  const ext = props.node.name.split('.').pop()?.toLowerCase() || '';
  if (isFolder.value) return Folder;
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'].includes(ext)) return FileImage;
  if (['zip', 'tar', 'gz', 'rar', '7z'].includes(ext)) return FileArchive;
  if (['html', 'css', 'js', 'ts', 'json', 'xml', 'py', 'java', 'cpp', 'rs'].includes(ext))
    return FileCode;
  if (['pdf', 'doc', 'docx', 'txt', 'md', 'rtf'].includes(ext)) return FileText;
  if (['mp4', 'avi', 'mkv', 'mov', 'webm'].includes(ext)) return FileVideo;
  if (['mp3', 'wav', 'flac', 'aac', 'ogg'].includes(ext)) return Music;
  return File;
});

function formatSize(bytes: number) {
  if (bytes === 0) return '—';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

function formatDate(epoch: number) {
  if (epoch === 0) return '—';
  const d = new Date(epoch * 1000);
  const now = new Date();
  const diff = Math.floor((now.getTime() - d.getTime()) / 1000);
  if (diff < 60) return 'just now';
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
  return d.toLocaleDateString();
}

function onClick(e: MouseEvent) {
  store.setFocus(props.node.path);
  if (e.ctrlKey || e.metaKey) {
    store.toggleSelected(props.node.path);
  } else if (e.shiftKey && store.focusedPath) {
    store.selectRange(store.focusedPath, props.node.path);
  } else {
    store.clearSelection();
    store.setSelected(props.node.path, true);
  }
}

function onDblClick() {
  if (isFolder.value) {
    store.navigateTo(props.node.path);
  }
}

function onContextMenu(e: MouseEvent) {
  e.preventDefault();
  store.setFocus(props.node.path);
  if (!store.selectedPaths.has(props.node.path)) {
    store.clearSelection();
    store.setSelected(props.node.path, true);
  }
  // emit event to FileTree / parent to open context menu
  emit('contextmenu', e, props.node);
}

const emit = defineEmits<{
  (e: 'contextmenu', event: MouseEvent, node: FileNode): void;
}>();
</script>

<template>
  <div
    class="file-tree-row group"
    :class="{
      selected: isSelected,
      focused: isFocused,
    }"
    tabindex="0"
    :aria-selected="isSelected"
    :aria-label="`${node.name} ${node.entryType}`"
    @click="onClick"
    @dblclick="onDblClick"
    @contextmenu="onContextMenu"
  >
    <ChevronRight
      v-if="isFolder"
      :size="16"
      class="text-theme-muted shrink-0 transition-transform duration-150"
    />
    <div v-else class="w-4 shrink-0" />

    <component
      :is="iconComponent"
      :size="18"
      class="shrink-0"
      :class="isFolder ? 'text-accent-emerald' : 'text-theme-secondary'"
    />

    <div class="file-tree-name">{{ node.name }}</div>

    <div class="file-tree-meta">
      <div class="file-tree-perms">{{ node.permissions }}</div>
      <div class="file-tree-date">{{ formatDate(node.modifiedEpochSecs) }}</div>
      <div class="file-tree-size">{{ formatSize(node.sizeBytes) }}</div>
    </div>
  </div>
</template>

<style scoped>
.file-tree-row {
  @apply flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-colors;
}

.file-tree-row:hover {
  @apply bg-theme-hover;
}

.file-tree-row.selected {
  @apply bg-accent-light;
}

.file-tree-row.focused {
  @apply outline outline-2 outline-accent-emerald/40 -outline-offset-2;
}

.file-tree-row:active {
  transform: scale(0.998);
}

.file-tree-name {
  @apply flex-1 text-sm text-theme-primary truncate;
}

.file-tree-meta {
  @apply hidden sm:flex items-center gap-4 text-[11px] text-theme-muted;
}

.file-tree-perms {
  @apply font-mono w-24;
}

.file-tree-date,
.file-tree-size {
  @apply w-20 text-right;
}

@media (prefers-reduced-motion: reduce) {
  .file-tree-row {
    transition: none;
  }
}
</style>
