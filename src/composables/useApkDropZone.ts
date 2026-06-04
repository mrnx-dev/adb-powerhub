import { readonly, ref } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { DragDropEvent } from '@tauri-apps/api/webview';
import { useAppsStore } from '@/stores/apps';
import { useDeviceStore } from '@/stores/device';
import { useToastStore } from '@/stores/toast';
import { useNavigationStore } from '@/stores/navigation';

const APK_EXTENSIONS = new Set(['apk', 'apks', 'xapk']);

function filterPathsByExt(paths: string[], extensions: Set<string>): string[] {
  return paths.filter((p) => {
    const ext = p.split('.').pop()?.toLowerCase();
    return ext && extensions.has(ext);
  });
}

function fileNameOf(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

// Module-level state (singleton — shared across all callers)
const isDragOver = ref(false);
const hasApkFiles = ref(false);
const isQueueProcessing = ref(false);
const queueProgress = ref<{ current: number; total: number } | null>(null);

// Listener lifecycle
let unlisten: (() => void) | null = null;
let initialized = false;
let dragOverTimer: ReturnType<typeof setTimeout> | null = null;

function clearDragState() {
  isDragOver.value = false;
  hasApkFiles.value = false;
  if (dragOverTimer) {
    clearTimeout(dragOverTimer);
    dragOverTimer = null;
  }
}

function resetDragTimer() {
  if (dragOverTimer) clearTimeout(dragOverTimer);
  // R1: Safety timeout — auto-hide overlay if leave/drop doesn't fire within 3s
  dragOverTimer = setTimeout(clearDragState, 3000);
}

async function processQueue(paths: string[]) {
  const appsStore = useAppsStore();
  const deviceStore = useDeviceStore();
  const toastStore = useToastStore();
  const navStore = useNavigationStore();

  isQueueProcessing.value = true;
  queueProgress.value = { current: 0, total: paths.length };

  const results: { success: boolean; filename: string; error?: string }[] = [];

  for (let i = 0; i < paths.length; i++) {
    // R2: Guard — device disconnected between files
    if (!deviceStore.connected) {
      toastStore.show('Device disconnected — install queue aborted', 'error');
      deviceStore.addLog('APK install queue aborted — device disconnected', 'error');
      break;
    }

    queueProgress.value = { current: i + 1, total: paths.length };
    deviceStore.addLog(`APK install started: ${fileNameOf(paths[i])}`, 'info');

    const result = await appsStore.installApkFromPath(paths[i]);
    results.push(result);

    if (result.success) {
      deviceStore.addLog(`APK installed: ${result.filename}`, 'info');
    } else {
      deviceStore.addLog(`APK install failed: ${result.filename} — ${result.error}`, 'error');
    }
  }

  isQueueProcessing.value = false;
  queueProgress.value = null;

  // Report results
  const succeeded = results.filter((r) => r.success).length;
  const failed = results.filter((r) => !r.success).length;

  if (failed === 0 && succeeded > 0) {
    toastStore.show(
      paths.length === 1
        ? `Installed ${results[0].filename}`
        : `Installed ${succeeded} app${succeeded > 1 ? 's' : ''}`,
      'success'
    );
  } else if (succeeded > 0 && failed > 0) {
    toastStore.show(`Installed ${succeeded} of ${results.length} — ${failed} failed`, 'error');
  } else if (failed > 0) {
    toastStore.show(
      results.length === 1
        ? `Install failed: ${results[0].error}`
        : `All ${results.length} install(s) failed`,
      'error'
    );
  }

  // Auto-refresh app list if on Apps page
  if (succeeded > 0 && navStore.currentPage === 'apps' && deviceStore.connected) {
    await appsStore.fetchApps();
  }
}

async function handleDragDropEvent(event: { payload: DragDropEvent }) {
  const appsStore = useAppsStore();
  const deviceStore = useDeviceStore();
  const toastStore = useToastStore();
  const payload = event.payload;

  switch (payload.type) {
    case 'enter': {
      const apkPaths = filterPathsByExt(payload.paths, APK_EXTENSIONS);
      if (apkPaths.length > 0) {
        hasApkFiles.value = true;
        isDragOver.value = true;
      }
      resetDragTimer();
      break;
    }

    case 'over': {
      // over doesn't carry paths — just keep overlay visible and reset safety timer (R1)
      if (isDragOver.value) {
        resetDragTimer();
      }
      break;
    }

    case 'leave': {
      clearDragState();
      break;
    }

    case 'drop': {
      clearDragState();
      const apkPaths = filterPathsByExt(payload.paths, APK_EXTENSIONS);

      if (apkPaths.length === 0) {
        toastStore.show('Not a valid APK file', 'error');
        return;
      }

      if (!deviceStore.connected) {
        toastStore.show('No device connected — connect a device first', 'error');
        return;
      }

      // R3 + R6: Mutual exclusion — reject if install already running
      if (appsStore.isInstalling || isQueueProcessing.value) {
        toastStore.show('An install is already in progress', 'error');
        return;
      }

      await processQueue(apkPaths);
      break;
    }
  }
}

/**
 * Read drag-drop state from anywhere.
 * App.vue must call initApkDropZone() in onMounted and destroyApkDropZone() in onBeforeUnmount.
 */
export function useApkDropZone() {
  return {
    isDragOver: readonly(isDragOver),
    hasApkFiles: readonly(hasApkFiles),
    isQueueProcessing: readonly(isQueueProcessing),
    queueProgress: readonly(queueProgress),
  };
}

/**
 * Register the Tauri native drag-drop listener. Call once in App.vue onMounted.
 */
export async function initApkDropZone() {
  if (initialized) return;
  initialized = true;
  try {
    unlisten = await getCurrentWebviewWindow().onDragDropEvent(handleDragDropEvent);
  } catch (e) {
    // R4: Non-critical — drag-drop is an enhancement, not core functionality
    console.warn('Failed to register drag-drop listener:', e);
    initialized = false;
  }
}

/**
 * Unregister the Tauri listener. Call once in App.vue onBeforeUnmount.
 */
export function destroyApkDropZone() {
  unlisten?.();
  unlisten = null;
  initialized = false;
  clearDragState();
  isQueueProcessing.value = false;
  queueProgress.value = null;
  if (dragOverTimer) {
    clearTimeout(dragOverTimer);
    dragOverTimer = null;
  }
}
