import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { load, Store } from "@tauri-apps/plugin-store";
import { useDeviceStore } from "./device";

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load("settings.json", { autoSave: true });
  }
  return storeInstance;
}

export const useSettingsStore = defineStore("settings", () => {
  const adbPath = ref("adb");
  const scrcpyPath = ref("");
  const autoDetectBinaries = ref(true);
  const stayOnTop = ref(false);
  const autoConnectOnLaunch = ref(false);
  const pollingInterval = ref(3);
  const videoQuality = ref<"low" | "medium" | "high" | "custom">("medium");
  const customBitRate = ref(4);
  const customMaxSize = ref(1080);
  const recordingFormat = ref<"mp4" | "mkv">("mkv");
  const screenshotSaveDir = ref("");
  const recordingSaveDir = ref("");

  const adbValid = ref(false);
  const adbVersion = ref("");
  const scrcpyValid = ref(false);
  const scrcpyVersion = ref("");
  const downloading = ref<"adb" | null>(null);
  const downloadProgress = ref({ read: 0, total: 0, percent: 0 });
  const downloadError = ref<string | null>(null);
  const downloadCancelled = ref(false);
  const downloadInfo = ref<{
    os: string;
    adb_download_url: string;
    adb_size_mb: string;
    scrcpy_link_url: string;
    scrcpy_link_label: string;
    scrcpy_install_hint: string;
  } | null>(null);
  const currentOs = ref("");

  const copiedHint = ref(false);

  const isWindows = computed(() => currentOs.value === "windows");
  const isMacos = computed(() => currentOs.value === "macos");
  const isLinux = computed(() => currentOs.value === "linux");

  async function init() {
    await loadSettings();
    await fetchDownloadInfo();
    await autoDetect();
    await validateAdb();
    await validateScrcpy();
    await syncPathsToRust();
    await syncToDeviceStore();
    await listenListeners();
  }

  async function autoDetect() {
    if (!autoDetectBinaries.value) return;
    if (adbPath.value === "adb") {
      try {
        const detected = await invoke<string | null>("settings_detect_adb");
        if (detected) {
          adbPath.value = detected;
          await saveSetting("adbPath", adbPath.value);
        }
      } catch {}
    }
    if (!scrcpyPath.value) {
      try {
        const status = await invoke<{ available: boolean; path: string | null }>("adb_find_scrcpy");
        if (status.available && status.path) {
          scrcpyPath.value = status.path;
          await saveSetting("scrcpyPath", scrcpyPath.value);
        }
      } catch {}
    }
  }

  async function syncPathsToRust() {
    try {
      if (adbPath.value) {
        await invoke("settings_set_adb_path", { path: adbPath.value });
      }
      if (scrcpyPath.value) {
        await invoke("settings_set_scrcpy_path", { path: scrcpyPath.value });
      }
    } catch (e) {
      console.error("Failed to sync paths to Rust:", e);
    }
  }

  async function syncToDeviceStore() {
    const deviceStore = useDeviceStore();
    deviceStore.scrcpyAvailable = scrcpyValid.value;
    deviceStore.scrcpyPath = scrcpyPath.value || null;
  }

  async function loadSettings() {
    try {
      const s = await getStore();
      const keys = [
        "adbPath", "scrcpyPath", "autoDetectBinaries", "stayOnTop",
        "autoConnectOnLaunch", "pollingInterval", "videoQuality",
        "customBitRate", "customMaxSize", "recordingFormat",
        "screenshotSaveDir", "recordingSaveDir",
      ];
      for (const key of keys) {
        const val = await s.get<string | number | boolean>(key);
        if (val !== null && val !== undefined) {
          (this as any)[key] = val;
        }
      }
    } catch {
      // First run, no saved settings
    }
  }

  async function saveSetting(key: string, value: unknown) {
    try {
      const s = await getStore();
      await s.set(key, value);
    } catch (e) {
      console.error("Failed to save setting:", e);
    }
  }

  async function fetchDownloadInfo() {
    try {
      const info = await invoke<{
        os: string;
        adb_download_url: string;
        adb_size_mb: string;
        scrcpy_link_url: string;
        scrcpy_link_label: string;
        scrcpy_install_hint: string;
      }>("settings_get_download_info");
      downloadInfo.value = info;
      currentOs.value = info.os;
    } catch {
      currentOs.value = "unknown";
    }
  }

  async function validateAdb() {
    try {
      const result = await invoke<{ valid: boolean; version: string; path: string }>(
        "settings_validate_adb",
        { path: adbPath.value }
      );
      adbValid.value = result.valid;
      adbVersion.value = result.version;
    } catch {
      adbValid.value = false;
      adbVersion.value = "";
    }
  }

  async function validateScrcpy() {
    if (!scrcpyPath.value) {
      scrcpyValid.value = false;
      scrcpyVersion.value = "";
      return;
    }
    try {
      const result = await invoke<{ valid: boolean; version: string; path: string }>(
        "settings_validate_scrcpy",
        { path: scrcpyPath.value }
      );
      scrcpyValid.value = result.valid;
      scrcpyVersion.value = result.version;
    } catch {
      scrcpyValid.value = false;
      scrcpyVersion.value = "";
    }
  }

  async function browseAdbPath() {
    const filters = isWindows.value
      ? [{ name: "Executable", extensions: ["exe"] }]
      : undefined;
    const selected = await dialogOpen({
      multiple: false,
      filters,
    });
    if (selected) {
      adbPath.value = selected as string;
      await saveSetting("adbPath", adbPath.value);
      await invoke("settings_set_adb_path", { path: adbPath.value });
      await validateAdb();
    }
  }

  async function browseScrcpyPath() {
    const filters = isWindows.value
      ? [{ name: "Executable", extensions: ["exe"] }]
      : undefined;
    const selected = await dialogOpen({
      multiple: false,
      filters,
    });
    if (selected) {
      scrcpyPath.value = selected as string;
      await saveSetting("scrcpyPath", scrcpyPath.value);
      await invoke("settings_set_scrcpy_path", { path: scrcpyPath.value });
      await validateScrcpy();
    }
  }

  async function browseScreenshotDir() {
    const selected = await dialogOpen({ directory: true, multiple: false });
    if (selected) {
      screenshotSaveDir.value = selected as string;
      await saveSetting("screenshotSaveDir", screenshotSaveDir.value);
    }
  }

  async function browseRecordingDir() {
    const selected = await dialogOpen({ directory: true, multiple: false });
    if (selected) {
      recordingSaveDir.value = selected as string;
      await saveSetting("recordingSaveDir", recordingSaveDir.value);
    }
  }

  async function downloadAdb() {
    downloading.value = "adb";
    downloadError.value = null;
    downloadCancelled.value = false;
    downloadProgress.value = { read: 0, total: 0, percent: 0 };

    try {
      const path = await invoke<string>("settings_download_adb");
      adbPath.value = path;
      await saveSetting("adbPath", adbPath.value);
      await validateAdb();
      downloading.value = null;
    } catch (e) {
      const msg = String(e);
      if (msg.includes("cancelled")) {
        downloadCancelled.value = true;
      } else {
        downloadError.value = msg;
      }
      downloading.value = null;
    }
  }

  async function cancelDownload() {
    try {
      await invoke("settings_cancel_download");
      downloadCancelled.value = true;
    } catch (e) {
      console.error("Cancel failed:", e);
    }
  }

  function openScrcpyLink() {
    if (downloadInfo.value) {
      shellOpen(downloadInfo.value.scrcpy_link_url);
    }
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedHint.value = true;
      setTimeout(() => { copiedHint.value = false; }, 2000);
    } catch {
      // Clipboard not available
    }
  }

  async function setStayOnTop(val: boolean) {
    stayOnTop.value = val;
    await saveSetting("stayOnTop", val);
    try {
      await getCurrentWindow().setAlwaysOnTop(val);
    } catch (e) {
      console.error("setAlwaysOnTop failed:", e);
    }
  }

  async function setAutoConnectOnLaunch(val: boolean) {
    autoConnectOnLaunch.value = val;
    await saveSetting("autoConnectOnLaunch", val);
  }

  async function setAutoDetectBinaries(val: boolean) {
    autoDetectBinaries.value = val;
    await saveSetting("autoDetectBinaries", val);
  }

  async function setPollingInterval(val: number) {
    pollingInterval.value = val;
    await saveSetting("pollingInterval", val);
    const deviceStore = useDeviceStore();
    if (deviceStore.connected) {
      deviceStore.startPolling();
    }
  }

  async function setVideoQuality(val: "low" | "medium" | "high" | "custom") {
    videoQuality.value = val;
    await saveSetting("videoQuality", val);
  }

  async function setCustomBitRate(val: number) {
    customBitRate.value = val;
    await saveSetting("customBitRate", val);
  }

  async function setCustomMaxSize(val: number) {
    customMaxSize.value = val;
    await saveSetting("customMaxSize", val);
  }

  async function setRecordingFormat(val: "mp4" | "mkv") {
    recordingFormat.value = val;
    await saveSetting("recordingFormat", val);
  }

  function getScrcpyArgs(): string[] {
    const args: string[] = [];
    switch (videoQuality.value) {
      case "low":
        args.push("--video-bit-rate", "2M", "--max-size", "720");
        break;
      case "medium":
        args.push("--video-bit-rate", "4M", "--max-size", "1080");
        break;
      case "high":
        args.push("--video-bit-rate", "8M");
        break;
      case "custom":
        args.push("--video-bit-rate", `${customBitRate.value}M`);
        if (customMaxSize.value > 0) {
          args.push("--max-size", String(customMaxSize.value));
        }
        break;
    }
    return args;
  }

  async function listenListeners() {
    await listen<{ type: string; read: number; total: number }>("download-progress", (event) => {
      if (event.payload.type === "adb") {
        downloadProgress.value = {
          read: event.payload.read,
          total: event.payload.total,
          percent: event.payload.total > 0
            ? Math.round((event.payload.read / event.payload.total) * 100)
            : 0,
        };
      }
    });

    await listen<{ type: string; path: string }>("download-complete", (event) => {
      if (event.payload.type === "adb") {
        adbPath.value = event.payload.path;
        downloading.value = null;
        validateAdb();
      }
    });

    await listen<{ type: string }>("download-cancelled", () => {
      downloading.value = null;
      downloadCancelled.value = true;
    });
  }

  return {
    adbPath, scrcpyPath, autoDetectBinaries, stayOnTop, autoConnectOnLaunch,
    pollingInterval, videoQuality, customBitRate, customMaxSize, recordingFormat,
    screenshotSaveDir, recordingSaveDir,
    adbValid, adbVersion, scrcpyValid, scrcpyVersion,
    downloading, downloadProgress, downloadError, downloadCancelled,
    downloadInfo, currentOs, copiedHint,
    isWindows, isMacos, isLinux,
    init, loadSettings, saveSetting, fetchDownloadInfo,
    validateAdb, validateScrcpy, autoDetect, syncPathsToRust, syncToDeviceStore,
    browseAdbPath, browseScrcpyPath, browseScreenshotDir, browseRecordingDir,
    downloadAdb, cancelDownload, openScrcpyLink, copyToClipboard,
    setStayOnTop, setAutoConnectOnLaunch, setAutoDetectBinaries,
    setPollingInterval, setVideoQuality, setCustomBitRate, setCustomMaxSize,
    setRecordingFormat, getScrcpyArgs,
  };
});