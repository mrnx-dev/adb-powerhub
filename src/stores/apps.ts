import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open as dialogOpen, ask } from '@tauri-apps/plugin-dialog';
import { useDeviceStore } from './device';
import { useToastStore } from './toast';

interface AppInfo {
  package_name: string;
  label: string;
  version_name: string;
  version_code: number;
  is_system: boolean;
  is_enabled: boolean;
  is_updated_system: boolean;
  code_path: string;
  data_dir: string;
  apk_size?: number;
}

function fileNameOf(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export const useAppsStore = defineStore('apps', () => {
  const deviceStore = useDeviceStore();
  const toast = useToastStore();

  const apps = ref<AppInfo[]>([]);
  const selectedPackage = ref<string | null>(null);
  const appDetail = ref<AppInfo | null>(null);
  const isLoading = ref(false);
  const isInstalling = ref(false);
  const isActioning = ref(false);
  const filter = ref<'all' | 'third_party' | 'system' | 'disabled'>('all');
  const searchQuery = ref('');
  const error = ref<string | null>(null);
  const icons = ref<Record<string, string>>({}); // pkg → data:image/png;base64,...
  const iconStates = ref<Record<string, 'loading' | 'loaded' | 'failed'>>({});
  const isLoadingIcons = ref(false);

  // Grid mode + sort
  const viewMode = ref<'list' | 'grid'>('list');
  const sortBy = ref<'name' | 'size'>('name');
  const allAppsCache = ref<AppInfo[]>([]);

  // Hover/peek + pin (Phase 2 prep)
  const hoveredPackage = ref<string | null>(null);
  const pinnedPackage = ref<string | null>(null);

  let unlistenIconReady: UnlistenFn | null = null;
  let unlistenFetchComplete: UnlistenFn | null = null;

  const failedIconCount = computed(
    () => Object.values(iconStates.value).filter((s) => s === 'failed').length
  );

  // Debounced search + sort
  const filteredApps = computed(() => {
    let result = apps.value;
    const q = searchQuery.value.toLowerCase().trim();
    if (q) {
      result = result.filter(
        (a) => a.package_name.toLowerCase().includes(q) || a.label.toLowerCase().includes(q)
      );
    }
    // Client-side sort
    if (sortBy.value === 'size') {
      result = [...result].sort((a, b) => (b.apk_size ?? -1) - (a.apk_size ?? -1));
    } else {
      result = [...result].sort((a, b) => a.label.localeCompare(b.label));
    }
    return result;
  });

  const appCount = computed(() => apps.value.length);

  // Filter counts from cached full list (accurate only after 'all' fetch)
  const filterCounts = computed(() => {
    const source = filter.value === 'all' ? apps.value : allAppsCache.value;
    return {
      all: source.length,
      third_party: source.filter((a) => !a.is_system).length,
      system: source.filter((a) => a.is_system).length,
      disabled: source.filter((a) => !a.is_enabled).length,
    };
  });

  async function fetchApps() {
    if (!deviceStore.connected) return;
    isLoading.value = true;
    error.value = null;
    try {
      apps.value = await invoke<AppInfo[]>('adb_list_apps', { filter: filter.value });
      // Cache full list for filter counts
      if (filter.value === 'all') {
        allAppsCache.value = apps.value;
      }
    } catch (e) {
      error.value = String(e);
      apps.value = [];
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchIcons() {
    if (!deviceStore.connected || apps.value.length === 0) return;

    isLoadingIcons.value = true;

    // Mark all as loading — UI shows skeleton immediately
    const newStates: Record<string, 'loading' | 'loaded' | 'failed'> = {};
    for (const a of apps.value) {
      newStates[a.package_name] = icons.value[a.package_name] ? 'loaded' : 'loading';
    }
    iconStates.value = newStates;

    // Set up event listeners for progressive icon delivery
    await setupIconListeners();

    // Fire and forget — backend spawns a background thread, icons arrive via events
    const packages = apps.value.map((a) => ({
      package_name: a.package_name,
      code_path: a.code_path,
      version_code: a.version_code,
    }));
    console.log('[apps] Starting background icon fetch for', packages.length, 'apps');

    try {
      await invoke('adb_fetch_icons', { packages });
    } catch (e) {
      const msg = String(e);
      if (msg.includes('already in progress')) {
        console.log('[apps] Icon fetch already in progress, skipping');
      } else {
        console.warn('[apps] Icon fetch failed:', e);
        isLoadingIcons.value = false;
      }
    }
  }

  async function setupIconListeners() {
    if (unlistenIconReady) unlistenIconReady();
    if (unlistenFetchComplete) unlistenFetchComplete();

    unlistenIconReady = await listen<{ package: string; base64: string | null }>(
      'icon-ready',
      (event) => {
        const { package: pkg, base64 } = event.payload;
        if (base64) {
          icons.value = { ...icons.value, [pkg]: `data:image/png;base64,${base64}` };
          iconStates.value = { ...iconStates.value, [pkg]: 'loaded' };
        } else {
          iconStates.value = { ...iconStates.value, [pkg]: 'failed' };
        }
      }
    );

    unlistenFetchComplete = await listen<{
      total: number;
      cached: number;
      extracted: number;
      failed: number;
      duration_secs: number;
    }>('icons-fetch-complete', () => {
      isLoadingIcons.value = false;
    });
  }

  let detailGeneration = 0;

  async function fetchAppDetail(packageName: string) {
    const gen = ++detailGeneration;

    // 1. Pre-populate instantly from list data (already has real label)
    const fromList = apps.value.find((a) => a.package_name === packageName);
    if (fromList) {
      appDetail.value = { ...fromList };
    }

    // 2. Background fetch for technical fields (version, paths, flags)
    try {
      const detail = await invoke<AppInfo>('adb_app_detail', { package: packageName });
      // Discard stale result if a newer fetch was triggered while awaiting
      if (gen !== detailGeneration) return;
      appDetail.value = {
        ...detail,
        label: fromList?.label ?? detail.label,
      };
    } catch (e) {
      if (gen !== detailGeneration) return;
      toast.show(`Failed to load detail: ${e}`, 'error');
      if (!appDetail.value) {
        appDetail.value = null;
      }
    }
  }

  async function installApkFromPath(
    path: string
  ): Promise<{ success: boolean; filename: string; error?: string }> {
    if (!deviceStore.connected) {
      return { success: false, filename: fileNameOf(path), error: 'No device connected' };
    }
    isInstalling.value = true;
    try {
      await invoke('adb_install_apk', { path });
      return { success: true, filename: fileNameOf(path) };
    } catch (e) {
      return { success: false, filename: fileNameOf(path), error: String(e) };
    } finally {
      isInstalling.value = false;
    }
  }

  async function installApk() {
    const selected = await dialogOpen({
      multiple: false,
      filters: [{ name: 'APK', extensions: ['apk'] }],
    });
    if (!selected) return;
    const result = await installApkFromPath(selected as string);
    if (result.success) {
      toast.show(`Installed ${result.filename}`, 'success');
      await fetchApps();
    } else {
      toast.show(`Install failed: ${result.error}`, 'error');
    }
  }

  async function uninstallApp(packageName: string, isSystem: boolean) {
    const msg = isSystem
      ? 'This is a system app. It will be disabled for the current user but can be re-enabled later. Continue?'
      : `Are you sure you want to uninstall ${packageName}?`;
    const confirmed = await ask(msg, {
      title: isSystem ? 'Disable System App' : 'Uninstall App',
      kind: 'warning',
      okLabel: isSystem ? 'Disable' : 'Uninstall',
      cancelLabel: 'Cancel',
    });
    if (!confirmed) return;

    isActioning.value = true;
    try {
      await invoke('adb_uninstall_app', { package: packageName, isSystem });
      toast.show(
        isSystem ? 'System app disabled for current user' : `Uninstalled ${packageName}`,
        'success'
      );
      if (selectedPackage.value === packageName) {
        appDetail.value = null;
        selectedPackage.value = null;
      }
      await fetchApps();
    } catch (e) {
      toast.show(`Failed: ${e}`, 'error');
    } finally {
      isActioning.value = false;
    }
  }

  async function clearApp(packageName: string) {
    const confirmed = await ask(
      `Are you sure you want to clear all data for ${packageName}? This cannot be undone.`,
      { title: 'Clear App Data', kind: 'warning', okLabel: 'Clear Data', cancelLabel: 'Cancel' }
    );
    if (!confirmed) return;

    isActioning.value = true;
    try {
      await invoke('adb_clear_app', { package: packageName });
      toast.show(`Data cleared for ${packageName}`, 'success');
    } catch (e) {
      toast.show(`Failed: ${e}`, 'error');
    } finally {
      isActioning.value = false;
    }
  }

  async function forceStopApp(packageName: string) {
    isActioning.value = true;
    try {
      await invoke('adb_force_stop_app', { package: packageName });
      toast.show(`Force stopped ${packageName}`, 'success');
    } catch (e) {
      toast.show(`Failed: ${e}`, 'error');
    } finally {
      isActioning.value = false;
    }
  }

  async function enableApp(packageName: string) {
    isActioning.value = true;
    try {
      await invoke('adb_enable_app', { package: packageName });
      toast.show(`Enabled ${packageName}`, 'success');
      await fetchApps();
      if (selectedPackage.value === packageName) {
        await fetchAppDetail(packageName);
      }
    } catch (e) {
      toast.show(`Failed: ${e}`, 'error');
    } finally {
      isActioning.value = false;
    }
  }

  async function disableApp(packageName: string) {
    const confirmed = await ask(
      `Disable ${packageName}? The app will be hidden from the launcher but can be re-enabled.`,
      { title: 'Disable App', kind: 'warning', okLabel: 'Disable', cancelLabel: 'Cancel' }
    );
    if (!confirmed) return;

    isActioning.value = true;
    try {
      await invoke('adb_disable_app', { package: packageName });
      toast.show(`Disabled ${packageName}`, 'success');
      await fetchApps();
      if (selectedPackage.value === packageName) {
        await fetchAppDetail(packageName);
      }
    } catch (e) {
      toast.show(`Failed: ${e}`, 'error');
    } finally {
      isActioning.value = false;
    }
  }

  function selectApp(packageName: string) {
    if (pinnedPackage.value === packageName) {
      // Unpin
      clearPin();
    } else {
      pinnedPackage.value = packageName;
      hoveredPackage.value = null;
      selectedPackage.value = packageName;
      fetchAppDetail(packageName);
    }
  }

  function clearPin() {
    pinnedPackage.value = null;
    selectedPackage.value = null;
    appDetail.value = null;
  }

  function hoverApp(packageName: string) {
    if (pinnedPackage.value) return;
    hoveredPackage.value = packageName;
  }

  function unhoverApp() {
    if (pinnedPackage.value) return;
    hoveredPackage.value = null;
  }

  function clearSelection() {
    clearPin();
    hoveredPackage.value = null;
  }

  // Preview data: detail when pinned, list data when hovering
  const previewApp = computed(() => {
    const pkg = pinnedPackage.value ?? hoveredPackage.value;
    if (!pkg) return null;
    if (pinnedPackage.value && appDetail.value) return appDetail.value;
    return apps.value.find((a) => a.package_name === pkg) ?? null;
  });

  function reset() {
    apps.value = [];
    allAppsCache.value = [];
    selectedPackage.value = null;
    appDetail.value = null;
    error.value = null;
    isLoading.value = false;
    isActioning.value = false;
    isInstalling.value = false;
    icons.value = {};
    iconStates.value = {};
    isLoadingIcons.value = false;
    if (unlistenIconReady) {
      unlistenIconReady();
      unlistenIconReady = null;
    }
    if (unlistenFetchComplete) {
      unlistenFetchComplete();
      unlistenFetchComplete = null;
    }
  }

  return {
    apps,
    selectedPackage,
    appDetail,
    isLoading,
    isInstalling,
    isActioning,
    filter,
    searchQuery,
    error,
    filteredApps,
    appCount,
    icons,
    iconStates,
    isLoadingIcons,
    failedIconCount,
    viewMode,
    sortBy,
    filterCounts,
    hoveredPackage,
    pinnedPackage,
    previewApp,
    hoverApp,
    unhoverApp,
    clearPin,
    fetchApps,
    fetchIcons,
    fetchAppDetail,
    installApk,
    installApkFromPath,
    uninstallApp,
    clearApp,
    forceStopApp,
    enableApp,
    disableApp,
    selectApp,
    clearSelection,
    reset,
  };
});
