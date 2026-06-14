import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, save, ask } from '@tauri-apps/plugin-dialog';
import { useToastStore } from './toast';
import { useDeviceStore } from './device';

export type FileEntryType = 'file' | 'folder' | 'symlink' | 'other';

export interface FileNode {
  name: string;
  path: string;
  entryType: FileEntryType;
  sizeBytes: number;
  modifiedEpochSecs: number;
  permissions: string;
}

export interface FileListResult {
  entries: FileNode[];
  totalCount: number;
  truncated: boolean;
}

interface PushResult {
  local_path: string;
  success: boolean;
  error?: string;
}

interface DeleteSummary {
  deleted: number;
  failed: number;
}

const DEFAULT_ROOT = '/sdcard';
const BATCH_MIN = 2;

export const useFileExplorerStore = defineStore('fileExplorer', () => {
  const toast = useToastStore();
  const deviceStore = useDeviceStore();

  const currentPath = ref(DEFAULT_ROOT);
  const allowedRoot = ref(DEFAULT_ROOT);
  const entries = ref<FileNode[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const totalCount = ref(0);
  const truncated = ref(false);
  const searchQuery = ref('');
  const focusedPath = ref<string | null>(null);
  const selectedPaths = ref<Set<string>>(new Set());
  const isPulling = ref(false);
  const isPushing = ref(false);
  const isDeleting = ref(false);
  const isRenaming = ref(false);
  const renameTarget = ref<string | null>(null);
  const renameValue = ref('');
  const transferProgress = ref<{ current: number; total: number } | null>(null);

  const filteredEntries = computed(() => {
    const q = searchQuery.value.trim().toLowerCase();
    if (!q) return entries.value;
    return entries.value.filter((e) => e.name.toLowerCase().includes(q));
  });

  const selectedEntries = computed(() =>
    entries.value.filter((e) => selectedPaths.value.has(e.path))
  );

  const hasSelection = computed(() => selectedPaths.value.size > 0);
  const showBatchBar = computed(() => selectedPaths.value.size >= BATCH_MIN);

  function setError(message: string | null) {
    error.value = message;
  }

  function clearSelection() {
    selectedPaths.value.clear();
  }

  function toggleSelected(path: string) {
    const next = new Set(selectedPaths.value);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    selectedPaths.value = next;
  }

  function setSelected(path: string, add: boolean) {
    const next = new Set(selectedPaths.value);
    if (add) next.add(path);
    else next.delete(path);
    selectedPaths.value = next;
  }

  function clearFocus() {
    focusedPath.value = null;
  }

  function setFocus(path: string | null) {
    focusedPath.value = path;
  }

  function selectRange(fromPath: string, toPath: string) {
    const visible = filteredEntries.value;
    const fromIdx = visible.findIndex((e) => e.path === fromPath);
    const toIdx = visible.findIndex((e) => e.path === toPath);
    if (fromIdx === -1 || toIdx === -1) return;
    const start = Math.min(fromIdx, toIdx);
    const end = Math.max(fromIdx, toIdx);
    const next = new Set(selectedPaths.value);
    for (let i = start; i <= end; i++) {
      next.add(visible[i].path);
    }
    selectedPaths.value = next;
  }

  async function listFiles(path?: string) {
    if (!deviceStore.connected) {
      setError('No device connected');
      entries.value = [];
      return;
    }

    const target = path ?? currentPath.value;
    isLoading.value = true;
    setError(null);
    try {
      const result = await invoke<FileListResult>('adb_list_files', {
        path: target,
        allowedRoot: allowedRoot.value,
      });
      entries.value = result.entries;
      totalCount.value = result.totalCount;
      truncated.value = result.truncated;
      currentPath.value = target;
    } catch (e) {
      const msg = String(e);
      setError(msg);
      deviceStore.addLog(`File Explorer list failed: ${msg}`, 'error');
    } finally {
      isLoading.value = false;
    }
  }

  async function navigateTo(path: string) {
    await listFiles(path);
    clearSelection();
    clearFocus();
  }

  async function navigateUp() {
    const p = currentPath.value.replace(/\/$/, '');
    const lastSlash = p.lastIndexOf('/');
    if (lastSlash <= 0) return;
    await navigateTo(p.slice(0, lastSlash) || '/');
  }

  function setSearch(q: string) {
    searchQuery.value = q;
  }

  async function pull(paths: string[]) {
    if (!deviceStore.connected || paths.length === 0) return;

    isPulling.value = true;
    transferProgress.value = { current: 0, total: paths.length };

    try {
      let localDest = '';
      if (paths.length === 1) {
        const picked = await save({ defaultPath: paths[0].split('/').pop() });
        if (!picked) return;
        localDest = picked;
      } else {
        const picked = await open({ directory: true });
        if (!picked) return;
        localDest = picked;
      }

      let succeeded = 0;
      for (let i = 0; i < paths.length; i++) {
        transferProgress.value = { current: i + 1, total: paths.length };
        try {
          await invoke('adb_pull', {
            devicePath: paths[i],
            localPath: localDest,
          });
          succeeded += 1;
        } catch (e) {
          deviceStore.addLog(`Pull failed ${paths[i]}: ${e}`, 'error');
        }
      }

      if (succeeded === paths.length) {
        toast.show(
          paths.length === 1 ? 'File pulled successfully' : `Pulled ${succeeded} items`,
          'success'
        );
      } else {
        toast.show(`Pulled ${succeeded} of ${paths.length}`, 'error');
      }
    } catch (e) {
      toast.show(`Pull failed: ${e}`, 'error');
    } finally {
      isPulling.value = false;
      transferProgress.value = null;
    }
  }

  async function push() {
    if (!deviceStore.connected) return;

    const picked = await open({ multiple: true });
    if (!picked || (Array.isArray(picked) && picked.length === 0)) return;

    const localPaths = Array.isArray(picked) ? picked : [picked];

    // Check overwrite for existing names
    const existingNames = new Set(entries.value.map((e) => e.name));
    const willOverwrite = localPaths.some((p) => existingNames.has(p.split(/[\\/]/).pop() || ''));
    if (willOverwrite) {
      const ok = await ask('Some files already exist. Overwrite?', {
        title: 'Overwrite?',
        kind: 'warning',
      });
      if (!ok) return;
    }

    isPushing.value = true;
    transferProgress.value = { current: 0, total: localPaths.length };

    try {
      const results = await invoke<PushResult[]>('adb_push', {
        localPaths,
        deviceDir: currentPath.value,
      });
      const failed = results.filter((r) => !r.success);
      if (failed.length === 0) {
        toast.show(`Pushed ${results.length} file(s)`, 'success');
      } else {
        toast.show(`Pushed ${results.length - failed.length} of ${results.length}`, 'error');
      }
      await listFiles();
    } catch (e) {
      toast.show(`Push failed: ${e}`, 'error');
    } finally {
      isPushing.value = false;
      transferProgress.value = null;
    }
  }

  async function deletePaths(paths: string[]) {
    if (!deviceStore.connected || paths.length === 0) return;

    const ok = await ask(`Delete ${paths.length} item(s)? This cannot be undone.`, {
      title: 'Confirm Delete',
      kind: 'warning',
    });
    if (!ok) return;

    isDeleting.value = true;
    try {
      const summary = await invoke<DeleteSummary>('adb_delete', {
        paths,
        allowedRoot: allowedRoot.value,
      });
      if (summary.failed === 0) {
        toast.show(`Deleted ${summary.deleted} item(s)`, 'success');
      } else {
        toast.show(`Deleted ${summary.deleted}, ${summary.failed} failed`, 'error');
      }
      clearSelection();
      await listFiles();
    } catch (e) {
      toast.show(`Delete failed: ${e}`, 'error');
    } finally {
      isDeleting.value = false;
    }
  }

  async function rename(oldPath: string, newName: string) {
    if (!deviceStore.connected) return;
    if (!newName || newName.includes('/') || newName.includes('\0')) {
      toast.show('Invalid name', 'error');
      return;
    }

    const parent = oldPath.slice(0, oldPath.lastIndexOf('/'));
    const newPath = `${parent}/${newName}`;

    isRenaming.value = true;
    try {
      await invoke('adb_rename', {
        oldPath,
        newPath,
        allowedRoot: allowedRoot.value,
      });
      toast.show('Renamed successfully', 'success');
      cancelRename();
      await listFiles();
    } catch (e) {
      toast.show(`Rename failed: ${e}`, 'error');
    } finally {
      isRenaming.value = false;
    }
  }

  function startRename(path: string) {
    const node = entries.value.find((e) => e.path === path);
    if (!node) return;
    renameTarget.value = path;
    renameValue.value = node.name;
  }

  function cancelRename() {
    renameTarget.value = null;
    renameValue.value = '';
  }

  async function commitRename() {
    if (!renameTarget.value) return;
    await rename(renameTarget.value, renameValue.value.trim());
  }

  async function createFolder(name: string) {
    if (!deviceStore.connected) return;
    if (!name || name.includes('/') || name.includes('\0')) {
      toast.show('Invalid folder name', 'error');
      return;
    }

    try {
      await invoke<string>('adb_mkdir', {
        parent: currentPath.value,
        name,
        allowedRoot: allowedRoot.value,
      });
      toast.show('Folder created', 'success');
      await listFiles();
    } catch (e) {
      toast.show(`Create folder failed: ${e}`, 'error');
    }
  }

  function init() {
    if (currentPath.value !== DEFAULT_ROOT) {
      currentPath.value = DEFAULT_ROOT;
    }
    listFiles();
  }

  return {
    currentPath,
    allowedRoot,
    entries,
    isLoading,
    error,
    totalCount,
    truncated,
    searchQuery,
    focusedPath,
    selectedPaths,
    isPulling,
    isPushing,
    isDeleting,
    isRenaming,
    transferProgress,
    filteredEntries,
    selectedEntries,
    hasSelection,
    showBatchBar,
    listFiles,
    navigateTo,
    navigateUp,
    setSearch,
    pull,
    push,
    deletePaths,
    rename,
    createFolder,
    init,
    startRename,
    cancelRename,
    commitRename,
    renameTarget,
    renameValue,
    clearSelection,
    toggleSelected,
    setSelected,
    clearFocus,
    setFocus,
    selectRange,
    setError,
  };
});
