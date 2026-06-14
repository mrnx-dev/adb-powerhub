<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { useFileExplorerStore } from '../stores/fileExplorer';
import { useDeviceStore } from '../stores/device';
import { Folder, Upload, Plus, Loader2, HardDrive } from '@lucide/vue';
import BreadcrumbBar from '../components/file/BreadcrumbBar.vue';
import SearchBar from '../components/file/SearchBar.vue';
import FileTree from '../components/file/FileTree.vue';
import BatchBar from '../components/file/BatchBar.vue';

const store = useFileExplorerStore();
const deviceStore = useDeviceStore();
const newFolderName = ref('');
const showNewFolder = ref(false);

async function handleCreateFolder() {
  if (!newFolderName.value.trim()) return;
  await store.createFolder(newFolderName.value.trim());
  newFolderName.value = '';
  showNewFolder.value = false;
}

async function handleRename() {
  await store.commitRename();
}

watch(
  () => store.renameTarget,
  (target) => {
    if (target) {
      showNewFolder.value = false;
    }
  }
);

onMounted(() => {
  store.init();
});
</script>

<template>
  <div tabindex="-1" class="flex flex-col h-full overflow-hidden outline-none bg-glow">
    <!-- Toolbar -->
    <div
      class="flex items-center gap-3 px-4 py-3 border-b border-theme-tertiary shrink-0 flex-wrap"
    >
      <BreadcrumbBar />
      <div class="flex-1" />
      <SearchBar />
      <button
        class="btn-pressable flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border border-theme-tertiary text-theme-secondary hover-subtle"
        :disabled="!deviceStore.connected || store.isPushing"
        @click="store.push()"
      >
        <Loader2 v-if="store.isPushing" :size="14" class="animate-spin" />
        <Upload v-else :size="14" />
        <span>Push</span>
      </button>
      <button
        class="btn-pressable flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border border-theme-tertiary text-theme-secondary hover-subtle"
        :disabled="!deviceStore.connected"
        @click="showNewFolder = true"
      >
        <Plus :size="14" />
        <span class="hidden sm:inline">New Folder</span>
      </button>
    </div>

    <!-- Error banner -->
    <div
      v-if="store.error"
      class="flex items-center justify-between px-4 py-2 bg-color-error-container border-b border-color-error text-color-error text-xs"
    >
      <span>{{ store.error }}</span>
      <button class="underline hover:text-color-error" @click="store.listFiles()">Retry</button>
    </div>

    <!-- Rename inline form -->
    <div
      v-if="store.renameTarget"
      class="flex items-center gap-2 px-4 py-2 border-b border-theme-tertiary bg-theme-card"
    >
      <input
        :value="store.renameValue"
        type="text"
        placeholder="New name"
        class="flex-1 min-w-0 px-3 py-1.5 rounded-md bg-theme-btn border border-theme-tertiary text-sm text-theme-primary focus:outline-none focus:border-accent-focus"
        @input="store.renameValue = ($event.target as HTMLInputElement).value"
        @keydown.enter="handleRename"
        @keydown.esc="store.cancelRename()"
      />
      <button
        class="btn-pressable px-3 py-1.5 rounded-lg text-xs bg-accent-emerald text-theme-inverse"
        @click="handleRename"
      >
        Rename
      </button>
      <button
        class="btn-pressable px-3 py-1.5 rounded-lg text-xs border border-theme-tertiary text-theme-secondary"
        @click="store.cancelRename()"
      >
        Cancel
      </button>
    </div>

    <!-- New folder inline form -->
    <div
      v-if="showNewFolder && !store.renameTarget"
      class="flex items-center gap-2 px-4 py-2 border-b border-theme-tertiary bg-theme-card"
    >
      <input
        v-model="newFolderName"
        type="text"
        placeholder="Folder name"
        class="flex-1 min-w-0 px-3 py-1.5 rounded-md bg-theme-btn border border-theme-tertiary text-sm text-theme-primary focus:outline-none focus:border-accent-focus"
        @keydown.enter="handleCreateFolder"
        @keydown.esc="showNewFolder = false"
      />
      <button
        class="btn-pressable px-3 py-1.5 rounded-lg text-xs bg-accent-emerald text-theme-inverse"
        @click="handleCreateFolder"
      >
        Create
      </button>
      <button
        class="btn-pressable px-3 py-1.5 rounded-lg text-xs border border-theme-tertiary text-theme-secondary"
        @click="showNewFolder = false"
      >
        Cancel
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-3 relative" tabindex="-1">
      <div v-if="!deviceStore.connected" class="empty-state">
        <div class="w-16 h-16 rounded-full bg-theme-card flex items-center justify-center mb-4">
          <HardDrive :size="32" class="text-theme-muted" />
        </div>
        <p class="text-theme-primary font-medium">No device connected</p>
        <p class="text-theme-muted text-sm mt-1">Connect a device to browse files</p>
        <button
          class="mt-4 btn-pressable px-4 py-2 rounded-lg bg-accent-emerald text-theme-inverse text-sm font-medium"
          @click="deviceStore.autoConnect()"
        >
          Reconnect
        </button>
      </div>

      <div v-else-if="store.isLoading" class="space-y-2">
        <div
          v-for="i in 12"
          :key="i"
          class="skeleton-row flex items-center gap-3 px-3 py-2.5 rounded-lg"
        >
          <div class="w-4 h-4 skeleton-box" />
          <div class="w-5 h-5 skeleton-box rounded" />
          <div class="w-32 h-4 skeleton-box" />
          <div class="flex-1" />
          <div class="w-20 h-3 skeleton-box" />
          <div class="w-16 h-3 skeleton-box" />
          <div class="w-12 h-3 skeleton-box" />
        </div>
      </div>

      <div v-else-if="store.filteredEntries.length === 0" class="empty-state">
        <div class="w-16 h-16 rounded-full bg-theme-card flex items-center justify-center mb-4">
          <Folder :size="32" class="text-theme-muted" />
        </div>
        <p class="text-theme-primary font-medium">No files here</p>
        <p class="text-theme-muted text-sm mt-1">
          This folder is empty or your search didn't match anything
        </p>
        <button
          class="mt-4 btn-pressable px-4 py-2 rounded-lg bg-accent-emerald text-theme-inverse text-sm font-medium"
          @click="store.push()"
        >
          Push Files
        </button>
      </div>

      <FileTree v-else />
    </div>

    <BatchBar />
  </div>
</template>

<style scoped>
.empty-state {
  @apply flex flex-col items-center justify-center h-full;
}

.skeleton-box {
  background: linear-gradient(
    90deg,
    var(--bg-card) 25%,
    rgba(255, 255, 255, 0.05) 50%,
    var(--bg-card) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  .skeleton-box {
    animation: none;
  }
}
</style>
