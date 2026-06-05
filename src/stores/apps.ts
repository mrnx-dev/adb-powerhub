import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
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

  // Debounced search
  const filteredApps = computed(() => {
    let result = apps.value;
    const q = searchQuery.value.toLowerCase().trim();
    if (q) {
      result = result.filter(
        (a) => a.package_name.toLowerCase().includes(q) || a.label.toLowerCase().includes(q)
      );
    }
    return result;
  });

  const appCount = computed(() => apps.value.length);

  async function fetchApps() {
    if (!deviceStore.connected) return;
    isLoading.value = true;
    error.value = null;
    try {
      apps.value = await invoke<AppInfo[]>('adb_list_apps', { filter: filter.value });
    } catch (e) {
      error.value = String(e);
      apps.value = [];
    } finally {
      isLoading.value = false;
    }
  }

  async function fetchAppDetail(packageName: string) {
    // 1. Pre-populate instantly from list data (already has real label)
    const fromList = apps.value.find((a) => a.package_name === packageName);
    if (fromList) {
      appDetail.value = { ...fromList };
    }

    // 2. Background fetch for technical fields (version, paths, flags)
    try {
      const detail = await invoke<AppInfo>('adb_app_detail', { package: packageName });
      // Merge: keep real label from list, overwrite other fields from detail
      appDetail.value = {
        ...detail,
        label: fromList?.label ?? detail.label,
      };
    } catch (e) {
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
    selectedPackage.value = packageName;
    fetchAppDetail(packageName);
  }

  function clearSelection() {
    selectedPackage.value = null;
    appDetail.value = null;
  }

  function reset() {
    apps.value = [];
    selectedPackage.value = null;
    appDetail.value = null;
    error.value = null;
    isLoading.value = false;
    isActioning.value = false;
    isInstalling.value = false;
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
    fetchApps,
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
