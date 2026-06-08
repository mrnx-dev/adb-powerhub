import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from './settings';
import { useToastStore } from './toast';

export interface ImageDimensions {
  width: number;
  height: number;
}

export interface ScreenshotFile {
  filename: string;
  path: string;
  size_bytes: number;
  created_iso: string;
  dimensions: ImageDimensions | null;
}

interface ScreenshotListResult {
  files: ScreenshotFile[];
  total_count: number;
  truncated: boolean;
}

export type SortMode = 'newest' | 'oldest' | 'largest';
export type FilterMode = 'all' | 'today' | 'week';

export const useScreenshotsStore = defineStore('screenshots', () => {
  // ── state ──
  const files = ref<ScreenshotFile[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const lastScanDir = ref<string | null>(null);
  const lastScanTime = ref<number | null>(null);
  const sortMode = ref<SortMode>('newest');
  const filterMode = ref<FilterMode>('all');
  const lightboxOpen = ref(false);
  const lightboxIndex = ref(0);

  // ── computed ──
  const displayedFiles = computed(() => {
    let result = [...files.value];

    // Filter by time range
    if (filterMode.value === 'today') {
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      result = result.filter((f) => new Date(f.created_iso) >= today);
    } else if (filterMode.value === 'week') {
      const weekAgo = new Date();
      weekAgo.setDate(weekAgo.getDate() - 7);
      result = result.filter((f) => new Date(f.created_iso) >= weekAgo);
    }

    // Sort
    if (sortMode.value === 'oldest') {
      result.sort((a, b) => new Date(a.created_iso).getTime() - new Date(b.created_iso).getTime());
    } else if (sortMode.value === 'largest') {
      result.sort((a, b) => b.size_bytes - a.size_bytes);
    }
    // 'newest' — already sorted by Rust, no re-sort needed

    return result;
  });

  const currentLightboxFile = computed(() => {
    if (!lightboxOpen.value) return null;
    return displayedFiles.value[lightboxIndex.value] ?? null;
  });

  const isEmpty = computed(() => files.value.length === 0 && !loading.value);
  const isEmptyFiltered = computed(
    () => files.value.length > 0 && displayedFiles.value.length === 0
  );

  // ── helpers ──
  function getSaveDir(): string {
    const settingsStore = useSettingsStore();
    return settingsStore.screenshotSaveDir || '';
  }

  // ── actions ──
  async function refresh(): Promise<void> {
    const dir = getSaveDir();

    // Cache: skip if same directory scanned < 30s ago
    if (
      lastScanDir.value === dir &&
      lastScanTime.value &&
      Date.now() - lastScanTime.value < 30_000
    ) {
      return;
    }

    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<ScreenshotListResult>('adb_list_screenshots', {
        dirPath: dir || null,
      });
      files.value = result.files;
      lastScanDir.value = dir;
      lastScanTime.value = Date.now();
    } catch (e) {
      error.value = String(e);
      // Preserve existing files on transient errors
    } finally {
      loading.value = false;
    }
  }

  async function deleteFile(path: string): Promise<void> {
    const saveDir = getSaveDir();
    const toast = useToastStore();
    try {
      await invoke('adb_delete_screenshot', { path, saveDir: saveDir || null });
      files.value = files.value.filter((f) => f.path !== path);
      // Close lightbox if the deleted file was being previewed
      if (lightboxOpen.value && currentLightboxFile.value?.path === path) {
        closeLightbox();
      }
      toast.show('Screenshot deleted', 'success');
    } catch (e) {
      toast.show(`Could not delete: ${e}`, 'error');
      throw e;
    }
  }

  function prependPath(path: string): void {
    const lastSep = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
    const filename = lastSep >= 0 ? path.substring(lastSep + 1) : path;
    const entry: ScreenshotFile = {
      filename,
      path,
      size_bytes: 0,
      created_iso: new Date().toISOString(),
      dimensions: null,
    };
    if (!files.value.some((f) => f.path === path)) {
      files.value.unshift(entry);
    }
  }

  async function openFile(path: string): Promise<void> {
    await invoke('open_folder', { path });
  }

  async function openFolder(path: string): Promise<void> {
    const lastSep = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
    const dir = lastSep >= 0 ? path.substring(0, lastSep) : path;
    await invoke('open_folder', { path: dir });
  }

  async function copyPath(path: string): Promise<void> {
    await navigator.clipboard.writeText(path);
    const toast = useToastStore();
    toast.show('Path copied to clipboard', 'success');
  }

  function openLightbox(index: number): void {
    lightboxIndex.value = Math.max(0, Math.min(index, displayedFiles.value.length - 1));
    lightboxOpen.value = true;
  }

  function closeLightbox(): void {
    lightboxOpen.value = false;
  }

  function nextImage(): void {
    const len = displayedFiles.value.length;
    if (len > 0) {
      lightboxIndex.value = (lightboxIndex.value + 1) % len;
    }
  }

  function prevImage(): void {
    const len = displayedFiles.value.length;
    if (len > 0) {
      lightboxIndex.value = (lightboxIndex.value - 1 + len) % len;
    }
  }

  function setSort(mode: SortMode): void {
    sortMode.value = mode;
  }

  function setFilter(mode: FilterMode): void {
    filterMode.value = mode;
  }

  return {
    files,
    loading,
    error,
    lastScanDir,
    lastScanTime,
    sortMode,
    filterMode,
    lightboxOpen,
    lightboxIndex,
    displayedFiles,
    currentLightboxFile,
    isEmpty,
    isEmptyFiltered,
    refresh,
    deleteFile,
    prependPath,
    openFile,
    openFolder,
    copyPath,
    openLightbox,
    closeLightbox,
    nextImage,
    prevImage,
    setSort,
    setFilter,
  };
});
