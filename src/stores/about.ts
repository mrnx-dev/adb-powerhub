import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AppInfo, DependencyStatus, DebugInfo, UpdateCheckResult } from '../types/about';

export const useAboutStore = defineStore('about', () => {
  const appInfo = ref<AppInfo | null>(null);
  const dependencies = ref<DependencyStatus[]>([]);
  const debugInfo = ref<DebugInfo | null>(null);
  const checkingForUpdates = ref(false);
  const updateResult = ref<UpdateCheckResult | null>(null);

  async function load() {
    await loadAppInfo();
    await loadDependencyStatus();
  }

  async function loadAppInfo() {
    try {
      appInfo.value = await invoke<AppInfo>('about_get_app_info');
    } catch (e) {
      console.error('Failed to load app info:', e);
      appInfo.value = null;
    }
  }

  async function loadDependencyStatus() {
    try {
      dependencies.value = await invoke<DependencyStatus[]>('about_get_dependency_status');
    } catch (e) {
      console.error('Failed to load dependency status:', e);
      dependencies.value = [];
    }
  }

  async function getDebugInfo(): Promise<DebugInfo | null> {
    try {
      const info = await invoke<DebugInfo>('about_get_debug_info');
      debugInfo.value = info;
      return info;
    } catch (e) {
      console.error('Failed to load debug info:', e);
      return null;
    }
  }

  async function checkForUpdates(): Promise<UpdateCheckResult | null> {
    if (checkingForUpdates.value) return null;
    checkingForUpdates.value = true;
    try {
      const result = await invoke<UpdateCheckResult>('about_check_for_updates');
      updateResult.value = result;
      return result;
    } catch (e) {
      console.error('Failed to check for updates:', e);
      updateResult.value = {
        available: false,
        version: null,
        url: null,
        message: 'Failed to check for updates. Please try again later.',
      };
      return updateResult.value;
    } finally {
      checkingForUpdates.value = false;
    }
  }

  return {
    appInfo,
    dependencies,
    debugInfo,
    checkingForUpdates,
    updateResult,
    load,
    loadAppInfo,
    loadDependencyStatus,
    getDebugInfo,
    checkForUpdates,
  };
});
