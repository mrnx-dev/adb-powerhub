<script setup lang="ts">
import { useSettingsStore } from '../../stores/settings';
import { invoke } from '@tauri-apps/api/core';
import { Link, ExternalLink, Download, XCircle, RefreshCw } from '@lucide/vue';

const store = useSettingsStore();

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

async function applyAdbPath() {
  await store.saveSetting('adbPath', store.adbPath);
  await invoke('settings_set_adb_path', { path: store.adbPath });
  await store.validateAdb();
}
</script>

<template>
  <section class="card-glass p-4">
    <div class="flex items-center gap-2 mb-4">
      <Link :size="16" class="text-accent-emerald" />
      <h2 class="font-sans text-xs font-semibold tracking-wider uppercase">Binary Paths</h2>
    </div>

    <!-- ADB Path -->
    <div class="mb-6">
      <div class="flex items-center justify-between mb-2">
        <label class="text-xs font-medium text-theme-secondary">ADB Path</label>
        <span v-if="store.adbValid" class="text-xs text-color-success"
          >✅ {{ store.adbVersion.split('\n')[0] }}</span
        >
        <span
          v-else-if="store.downloading !== 'adb' && !store.downloadCancelled"
          class="text-xs text-color-error"
          >⚠ Not Found</span
        >
      </div>
      <div class="flex items-center gap-3">
        <input
          v-model="store.adbPath"
          type="text"
          class="flex-1 bg-theme-input border border-theme-secondary rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-50"
          @blur="applyAdbPath"
        />
        <button
          class="btn-pressable px-4 py-2 rounded-lg bg-theme-btn border border-theme-secondary text-sm hover-accent"
          @click="store.browseAdbPath"
        >
          Browse
        </button>
      </div>

      <!-- ADB Download (in-app) -->
      <div v-if="!store.adbValid && store.downloading !== 'adb'" class="mt-3">
        <!-- Download error -->
        <div
          v-if="store.downloadError"
          class="bg-color-error-container border border-color-error rounded-xl p-3 mb-2"
        >
          <p class="text-xs text-color-error mb-2">✗ Download failed: {{ store.downloadError }}</p>
          <button
            class="btn-pressable flex items-center gap-2 px-4 py-2 rounded-lg btn-primary text-xs font-semibold"
            @click="store.downloadAdb"
          >
            <RefreshCw :size="14" /> Retry Download
          </button>
        </div>

        <!-- Cancelled -->
        <div
          v-else-if="store.downloadCancelled"
          class="bg-color-warning-container border border-color-warning rounded-xl p-3 mb-2"
        >
          <p class="text-xs text-color-warning mb-2">Download cancelled</p>
          <button
            class="btn-pressable flex items-center gap-2 px-4 py-2 rounded-lg btn-primary text-xs font-semibold"
            @click="store.downloadAdb"
          >
            <RefreshCw :size="14" /> Retry Download
          </button>
        </div>

        <!-- Download prompt -->
        <div v-else class="bg-color-warning-container border border-color-warning rounded-xl p-3">
          <p class="text-xs text-color-warning mb-3">⚠ ADB not found on your system</p>
          <button
            class="btn-pressable flex items-center gap-2 px-4 py-2.5 rounded-lg btn-primary text-xs font-semibold"
            @click="store.downloadAdb"
          >
            <Download :size="14" />
            Download ADB for
            {{ store.currentOs.charAt(0).toUpperCase() + store.currentOs.slice(1) }}
          </button>
          <p class="text-[10px] text-theme-muted mt-2">
            Google Platform Tools · ~{{ store.downloadInfo?.adb_size_mb || '15' }}MB · Extracts to
            app data directory
          </p>
        </div>
      </div>

      <!-- ADB Download progress -->
      <div
        v-if="store.downloading === 'adb'"
        class="mt-3 bg-theme-btn border border-theme-secondary rounded-xl p-3"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="text-xs text-theme-secondary"
            >⬇ Downloading ADB... {{ store.downloadProgress.percent }}%</span
          >
          <span class="text-[10px] text-theme-muted">
            {{ formatBytes(store.downloadProgress.read) }} /
            {{ formatBytes(store.downloadProgress.total) }}
          </span>
        </div>
        <div class="progress-bar-track mb-3">
          <div
            class="progress-bar-fill transition-all duration-300"
            :style="{ width: store.downloadProgress.percent + '%' }"
          ></div>
        </div>
        <button
          class="btn-pressable flex items-center gap-2 px-4 py-2 rounded-lg bg-color-error-container border border-color-error text-color-error text-xs font-semibold hover:bg-color-error-container"
          @click="store.cancelDownload"
        >
          <XCircle :size="14" /> Cancel Download
        </button>
      </div>

      <!-- Update button (when adb found) -->
      <div v-if="store.adbValid && !store.downloading" class="mt-3">
        <button
          class="btn-pressable flex items-center gap-2 px-3 py-1.5 rounded-lg bg-theme-btn border border-theme-secondary text-xs text-theme-secondary hover-accent"
          @click="store.downloadAdb"
        >
          <Download :size="12" /> Update ADB
        </button>
      </div>
    </div>

    <!-- Scrcpy Path -->
    <div>
      <div class="flex items-center justify-between mb-2">
        <label class="text-xs font-medium text-theme-secondary">Scrcpy Path</label>
        <span v-if="store.scrcpyValid" class="text-xs text-color-success"
          >✅ {{ store.scrcpyVersion }}</span
        >
        <span v-else class="text-xs text-color-error">⚠ Not Found</span>
      </div>
      <div class="flex items-center gap-3">
        <input
          v-model="store.scrcpyPath"
          type="text"
          class="flex-1 bg-theme-input border border-theme-secondary rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-50"
          @blur="
            store.saveSetting('scrcpyPath', store.scrcpyPath);
            store.validateScrcpy();
          "
        />
        <button
          class="btn-pressable px-4 py-2 rounded-lg bg-theme-btn border border-theme-secondary text-sm hover-accent"
          @click="store.browseScrcpyPath"
        >
          Browse
        </button>
      </div>

      <!-- Scrcpy download link (recommendation only) -->
      <div
        v-if="!store.scrcpyValid"
        class="mt-3 bg-color-warning-container border border-color-warning rounded-xl p-3"
      >
        <p class="text-xs text-color-warning mb-3">⚠ Scrcpy not found</p>
        <button
          class="btn-pressable flex items-center gap-2 px-4 py-2.5 rounded-lg btn-primary text-xs font-semibold"
          @click="store.openScrcpyLink"
        >
          <ExternalLink :size="14" />
          {{ store.downloadInfo?.scrcpy_link_label || 'Download Scrcpy' }}
        </button>
        <div v-if="store.downloadInfo?.scrcpy_install_hint" class="mt-2">
          <p class="text-[10px] text-theme-muted mb-1">
            {{ store.downloadInfo.scrcpy_install_hint }}
          </p>
          <button
            class="btn-pressable text-[10px] text-accent-emerald hover:underline"
            @click="store.copyToClipboard(store.downloadInfo!.scrcpy_install_hint)"
          >
            {{ store.copiedHint ? 'Copied!' : 'Copy command' }}
          </button>
        </div>
      </div>

      <!-- Visit link (when scrcpy found) -->
      <div v-if="store.scrcpyValid && store.downloadInfo" class="mt-3">
        <button
          class="btn-pressable flex items-center gap-2 px-3 py-1.5 rounded-lg bg-theme-btn border border-theme-secondary text-xs text-theme-secondary hover-accent"
          @click="store.openScrcpyLink"
        >
          <ExternalLink :size="12" /> Visit Scrcpy Page
        </button>
      </div>
    </div>
  </section>
</template>
