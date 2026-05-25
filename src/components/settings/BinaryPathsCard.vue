<script setup lang="ts">
import { useSettingsStore } from "../../stores/settings";
import { invoke } from "@tauri-apps/api/core";
import { Link, ExternalLink, Download, XCircle, RefreshCw } from "lucide-vue-next";

const store = useSettingsStore();

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

async function applyAdbPath() {
  await store.saveSetting("adbPath", store.adbPath);
  await invoke("settings_set_adb_path", { path: store.adbPath });
  await store.validateAdb();
}
</script>

<template>
  <section class="card-glass border border-card-border rounded-2xl p-4">
    <div class="flex items-center gap-2 mb-4">
      <Link :size="16" class="text-accent-emerald" />
      <h2 class="text-xs font-bold uppercase tracking-widest">Binary Paths</h2>
    </div>

    <!-- ADB Path -->
    <div class="mb-6">
      <div class="flex items-center justify-between mb-2">
        <label class="text-xs font-medium text-gray-400">ADB Path</label>
        <span v-if="store.adbValid" class="text-xs text-green-400">✅ {{ store.adbVersion.split('\n')[0] }}</span>
        <span v-else-if="store.downloading !== 'adb' && !store.downloadCancelled" class="text-xs text-red-400">⚠ Not Found</span>
      </div>
      <div class="flex items-center gap-3">
        <input v-model="store.adbPath" @blur="applyAdbPath" type="text"
          class="flex-1 bg-black/40 border border-white/10 rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-emerald/50" />
        <button @click="store.browseAdbPath"
          class="px-4 py-2 rounded-lg bg-white/5 border border-white/10 text-sm hover:bg-white/10 hover:border-accent-emerald/50 transition-all">
          Browse
        </button>
      </div>

      <!-- ADB Download (in-app) -->
      <div v-if="!store.adbValid && store.downloading !== 'adb'" class="mt-3">
        <!-- Download error -->
        <div v-if="store.downloadError" class="bg-red-500/10 border border-red-500/20 rounded-xl p-3 mb-2">
          <p class="text-xs text-red-400 mb-2">✗ Download failed: {{ store.downloadError }}</p>
          <button @click="store.downloadAdb"
            class="flex items-center gap-2 px-4 py-2 rounded-lg bg-accent-emerald hover:bg-accent-emerald-hover text-white text-xs font-semibold transition-colors">
            <RefreshCw :size="14" /> Retry Download
          </button>
        </div>

        <!-- Cancelled -->
        <div v-else-if="store.downloadCancelled" class="bg-yellow-500/10 border border-yellow-500/20 rounded-xl p-3 mb-2">
          <p class="text-xs text-yellow-400 mb-2">Download cancelled</p>
          <button @click="store.downloadAdb"
            class="flex items-center gap-2 px-4 py-2 rounded-lg bg-accent-emerald hover:bg-accent-emerald-hover text-white text-xs font-semibold transition-colors">
            <RefreshCw :size="14" /> Retry Download
          </button>
        </div>

        <!-- Download prompt -->
        <div v-else class="bg-yellow-500/10 border border-yellow-500/20 rounded-xl p-3">
          <p class="text-xs text-yellow-400 mb-3">⚠ ADB not found on your system</p>
          <button @click="store.downloadAdb"
            class="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-accent-emerald hover:bg-accent-emerald-hover text-white text-xs font-semibold transition-colors shadow-lg shadow-emerald-500/20">
            <Download :size="14" />
            Download ADB for {{ store.currentOs.charAt(0).toUpperCase() + store.currentOs.slice(1) }}
          </button>
          <p class="text-[10px] text-gray-500 mt-2">
            Google Platform Tools · ~{{ store.downloadInfo?.adb_size_mb || "15" }}MB · Extracts to app data directory
          </p>
        </div>
      </div>

      <!-- ADB Download progress -->
      <div v-if="store.downloading === 'adb'" class="mt-3 bg-white/5 border border-white/10 rounded-xl p-3">
        <div class="flex items-center justify-between mb-2">
          <span class="text-xs text-gray-300">⬇ Downloading ADB... {{ store.downloadProgress.percent }}%</span>
          <span class="text-[10px] text-gray-500">
            {{ formatBytes(store.downloadProgress.read) }} / {{ formatBytes(store.downloadProgress.total) }}
          </span>
        </div>
        <div class="w-full bg-gray-700 rounded-full h-2 mb-3">
          <div class="bg-accent-emerald h-2 rounded-full transition-all duration-300"
            :style="{ width: store.downloadProgress.percent + '%' }"></div>
        </div>
        <button @click="store.cancelDownload"
          class="flex items-center gap-2 px-4 py-2 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-xs font-semibold hover:bg-red-500/20 transition-colors">
          <XCircle :size="14" /> Cancel Download
        </button>
      </div>

      <!-- Update button (when adb found) -->
      <div v-if="store.adbValid && !store.downloading" class="mt-3">
        <button @click="store.downloadAdb"
          class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 text-xs text-gray-400 hover:border-accent-emerald/50 hover:bg-white/10 hover:text-accent-emerald transition-all">
          <Download :size="12" /> Update ADB
        </button>
      </div>
    </div>

    <!-- Scrcpy Path -->
    <div>
      <div class="flex items-center justify-between mb-2">
        <label class="text-xs font-medium text-gray-400">Scrcpy Path</label>
        <span v-if="store.scrcpyValid" class="text-xs text-green-400">✅ {{ store.scrcpyVersion }}</span>
        <span v-else class="text-xs text-red-400">⚠ Not Found</span>
      </div>
      <div class="flex items-center gap-3">
        <input v-model="store.scrcpyPath" @blur="store.saveSetting('scrcpyPath', store.scrcpyPath); store.validateScrcpy()" type="text"
          class="flex-1 bg-black/40 border border-white/10 rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-emerald/50" />
        <button @click="store.browseScrcpyPath"
          class="px-4 py-2 rounded-lg bg-white/5 border border-white/10 text-sm hover:bg-white/10 hover:border-accent-emerald/50 transition-all">
          Browse
        </button>
      </div>

      <!-- Scrcpy download link (recommendation only) -->
      <div v-if="!store.scrcpyValid" class="mt-3 bg-yellow-500/10 border border-yellow-500/20 rounded-xl p-3">
        <p class="text-xs text-yellow-400 mb-3">⚠ Scrcpy not found</p>
        <button @click="store.openScrcpyLink"
          class="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-accent-emerald hover:bg-accent-emerald-hover text-white text-xs font-semibold transition-colors shadow-lg shadow-emerald-500/20">
          <ExternalLink :size="14" />
          {{ store.downloadInfo?.scrcpy_link_label || "Download Scrcpy" }}
        </button>
        <div v-if="store.downloadInfo?.scrcpy_install_hint" class="mt-2">
          <p class="text-[10px] text-gray-500 mb-1">{{ store.downloadInfo.scrcpy_install_hint }}</p>
          <button @click="store.copyToClipboard(store.downloadInfo!.scrcpy_install_hint)"
            class="text-[10px] text-accent-emerald hover:underline">
            {{ store.copiedHint ? "Copied!" : "Copy command" }}
          </button>
        </div>
      </div>

      <!-- Visit link (when scrcpy found) -->
      <div v-if="store.scrcpyValid && store.downloadInfo" class="mt-3">
        <button @click="store.openScrcpyLink"
          class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10 text-xs text-gray-400 hover:border-accent-emerald/50 hover:bg-white/10 hover:text-accent-emerald transition-all">
          <ExternalLink :size="12" /> Visit Scrcpy Page
        </button>
      </div>
    </div>

    <!-- Auto-detect toggle -->
    <div class="mt-6 pt-4 border-t border-white/5">
      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Auto-detect binaries on launch</span>
          <p class="text-[10px] text-gray-500">Scan system PATH for adb and scrcpy on startup</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer">
          <input type="checkbox" class="sr-only peer" v-model="store.autoDetectBinaries"
            @change="store.setAutoDetectBinaries(store.autoDetectBinaries)" />
          <div class="w-9 h-5 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-accent-emerald"></div>
        </label>
      </div>
    </div>
  </section>
</template>