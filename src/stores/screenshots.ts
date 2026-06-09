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

  // ── thumbnail + full-res caches (LRU eviction, inflight dedup) ──
  const thumbnailCache = ref<Map<string, string>>(new Map());
  const imageCache = ref<Map<string, string>>(new Map());
  const inflightThumbnails = ref<Map<string, Promise<string>>>(new Map());
  const inflightLoads = ref<Map<string, Promise<string>>>(new Map());

  // ── truncation + UX state ──
  const totalCount = ref(0);
  const truncationBannerDismissed = ref(false);
  const lightboxSourceRect = ref<DOMRect | null>(null);

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

  const isTruncated = computed(
    () => totalCount.value > files.value.length && files.value.length > 0
  );

  // ── thumbnail + full-res cache actions ──
  async function loadThumbnail(path: string): Promise<string> {
    // Validate path is in displayedFiles
    if (!displayedFiles.value.some((f) => f.path === path)) {
      throw new Error('Path not in displayed files');
    }
    // Check cache (LRU touch: delete + re-set)
    if (thumbnailCache.value.has(path)) {
      const val = thumbnailCache.value.get(path)!;
      thumbnailCache.value.delete(path);
      thumbnailCache.value.set(path, val);
      return val;
    }
    // Check inflight dedup
    if (inflightThumbnails.value.has(path)) {
      return inflightThumbnails.value.get(path)!;
    }
    // Fetch thumbnail
    const promise = invoke<string | null>('adb_get_thumbnail', { path })
      .then((result) => {
        if (result) {
          const url = `data:image/jpeg;base64,${result}`;
          thumbnailCache.value.set(path, url);
          // LRU evict if over limit
          if (thumbnailCache.value.size > 100) {
            const firstKey = thumbnailCache.value.keys().next().value;
            if (firstKey !== undefined) thumbnailCache.value.delete(firstKey);
          }
        }
        inflightThumbnails.value.delete(path);
        return result ?? '';
      })
      .catch(() => {
        inflightThumbnails.value.delete(path);
        return '';
      });
    inflightThumbnails.value.set(path, promise);
    return promise;
  }

  async function loadImage(path: string): Promise<string> {
    // Check cache (LRU touch)
    if (imageCache.value.has(path)) {
      const val = imageCache.value.get(path)!;
      imageCache.value.delete(path);
      imageCache.value.set(path, val);
      return val;
    }
    // Check inflight dedup
    if (inflightLoads.value.has(path)) {
      return inflightLoads.value.get(path)!;
    }
    // Fetch full-res base64
    const promise = invoke<string>('adb_read_file_base64', { path })
      .then((base64) => {
        const url = `data:image/png;base64,${base64}`;
        imageCache.value.set(path, url);
        // LRU evict if over limit
        if (imageCache.value.size > 30) {
          const firstKey = imageCache.value.keys().next().value;
          if (firstKey !== undefined) imageCache.value.delete(firstKey);
        }
        inflightLoads.value.delete(path);
        return url;
      })
      .catch(() => {
        inflightLoads.value.delete(path);
        return '';
      });
    inflightLoads.value.set(path, promise);
    return promise;
  }

  function clearCaches() {
    thumbnailCache.value.clear();
    imageCache.value.clear();
    inflightThumbnails.value.clear();
    inflightLoads.value.clear();
  }

  // ── helpers ──
  function getSaveDir(): string {
    const settingsStore = useSettingsStore();
    return settingsStore.screenshotSaveDir || '';
  }

  // ── actions ──
  async function refresh(dirPath?: string, force?: boolean): Promise<void> {
    const dir = dirPath ?? getSaveDir();

    // Cache: skip if same directory scanned < 30s ago (unless forced)
    if (
      !force &&
      lastScanDir.value === dir &&
      lastScanTime.value &&
      Date.now() - lastScanTime.value < 30_000
    ) {
      return;
    }

    loading.value = true;
    error.value = null;
    clearCaches();
    truncationBannerDismissed.value = false;
    try {
      const result = await invoke<ScreenshotListResult>('adb_list_screenshots', {
        dirPath: dir || null,
      });
      files.value = result.files;
      totalCount.value = result.total_count;
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

  function addFile(entry: ScreenshotFile): void {
    if (!files.value.some((f) => f.path === entry.path)) {
      files.value.push(entry);
      // Sort newest first
      files.value.sort((a, b) => b.created_iso.localeCompare(a.created_iso));
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

  function openLightbox(index: number, sourceRect?: DOMRect): void {
    lightboxIndex.value = Math.max(0, Math.min(index, displayedFiles.value.length - 1));
    lightboxOpen.value = true;
    lightboxSourceRect.value = sourceRect ?? null;
  }

  function closeLightbox(): void {
    lightboxOpen.value = false;
    lightboxSourceRect.value = null;
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

  // Retry a failed thumbnail — removes from cache and re-fetches
  // Debounce: ignore calls within 500ms of the last retry for the same path
  const _lastRetryTime = new Map<string, number>();
  function retryThumbnail(path: string): Promise<string> {
    const now = Date.now();
    const lastTime = _lastRetryTime.get(path) ?? 0;
    if (now - lastTime < 500) return Promise.resolve(''); // debounce
    _lastRetryTime.set(path, now);
    thumbnailCache.value.delete(path);
    return loadThumbnail(path);
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
    lightboxSourceRect,
    totalCount,
    truncationBannerDismissed,
    thumbnailCache,
    imageCache,
    displayedFiles,
    currentLightboxFile,
    isEmpty,
    isEmptyFiltered,
    isTruncated,
    refresh,
    deleteFile,
    prependPath,
    addFile,
    openFile,
    openFolder,
    copyPath,
    openLightbox,
    closeLightbox,
    nextImage,
    prevImage,
    setSort,
    setFilter,
    loadThumbnail,
    loadImage,
    clearCaches,
    retryThumbnail,
  };
});
