import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ask } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "./settings";
import { useToastStore } from "./toast";
import { useConnectionHistoryStore } from "./connectionHistory";

export const useDeviceStore = defineStore("device", () => {
  const toast = useToastStore();
  const connected = ref(false);
  const connecting = ref(false);
  const ipAddress = ref("");
  const port = ref(5555);
  const deviceId = ref("");
  const connectMethod = ref<"manual" | "wifi" | "pairing">("manual");
  const transport = ref<"usb" | "wifi">("usb");
  const autoConnectStatus = ref<"idle" | "detecting_usb" | "enabling_tcp" | "detecting_ip" | "connecting_tcp" | "connected" | "error">("idle");

  const batteryLevel = ref(0);
  const batteryStatus = ref("Unknown");
  const batteryHealth = ref("Unknown");
  const batteryTemp = ref(0);
  const batteryPlugged = ref(false);

  const cpuUsage = ref(0);
  const model = ref("—");
  const androidVersion = ref("—");
  const sdkVersion = ref("—");

  const logs = ref<string[]>([]);
  const commandInput = ref("");

  const scrcpyAvailable = ref(false);
  const scrcpyPath = ref<string | null>(null);
  const mirroring = ref(false);
  const recordingScreen = ref(false);

  const wifiEnabled = ref(false);
  const dataEnabled = ref(false);
  const airplaneEnabled = ref(false);
  const bluetoothEnabled = ref(false);
  const showTapsEnabled = ref(false);
  const layoutBoundsEnabled = ref(false);
  const stayAwakeEnabled = ref(false);
  const brightness = ref(128);
  const textInput = ref("");
  const showRebootMenu = ref(false);

  let pollTimeout: ReturnType<typeof setTimeout> | null = null;
  let pollFailCount = 0;
  let isPolling = false;
  let pollCount = 0;

  const isLoadingStats = ref(false);
  const isInitialLoad = ref(true);

  let cancelCurrentConnect: (() => void) | null = null;

  const MAX_RETRIES = 3;
  const RETRY_DELAY_MS = 2000;

  const batteryColor = computed(() => {
    if (batteryLevel.value > 60) return "text-color-success";
    if (batteryLevel.value > 20) return "text-color-warning";
    return "text-color-error";
  });

  const MAX_LOG_LENGTH = 500;

  function addLog(message: string, type: "info" | "error" | "success" = "info") {
    const timestamp = new Date().toLocaleTimeString();
    const prefix = type === "error" ? "✗" : type === "success" ? "✓" : "→";
    const truncated = message.length > MAX_LOG_LENGTH ? message.slice(0, MAX_LOG_LENGTH) + "..." : message;
    logs.value.push(`[${timestamp}] ${prefix} ${truncated}`);
    if (logs.value.length > 500) logs.value.shift();
  }

  function clearLogs() {
    logs.value = [];
  }

  function resetStats() {
    batteryLevel.value = 0;
    batteryStatus.value = "Unknown";
    batteryHealth.value = "Unknown";
    batteryTemp.value = 0;
    batteryPlugged.value = false;
    cpuUsage.value = 0;
    model.value = "—";
    androidVersion.value = "—";
    sdkVersion.value = "—";
  }

  async function onConnected(deviceIdStr: string, method: "manual" | "wifi" | "pairing") {
    connected.value = true;
    deviceId.value = deviceIdStr;
    connectMethod.value = method;
    autoConnectStatus.value = "connected";
    pollFailCount = 0;
    addLog(`Connected to ${deviceIdStr}`, "success");
    toast.show("Device connected", "success");
    await pollStats();
    startPolling();
    syncToggles();
    // save after pollStats so model / version info is populated
    const history = useConnectionHistoryStore();
    const deviceIp = ipAddress.value || deviceIdStr.split(":")[0];
    const devicePort = port.value || parseInt(deviceIdStr.split(":")[1] || "5555");
    const methodKey = method === "pairing" ? "pairing" as const : "wifi" as const;
    history.save({
      id: history.deviceId(deviceIp, devicePort),
      ip: deviceIp,
      port: devicePort,
      label: model.value || deviceIp,
      model: model.value,
      lastConnected: new Date().toISOString(),
      method: methodKey,
    });
  }

  async function connectWithPort(targetIp: string, targetPort: number) {
    connecting.value = true;
    try {
      await invoke<string>("adb_connect_port", { ip: targetIp, port: targetPort });
      try { await onConnected(`${targetIp}:${targetPort}`, "manual"); } catch (e) {
        addLog(`Post-connect setup failed: ${e}`, "error");
      }
    } catch (e) {
      addLog(String(e), "error");
      toast.show("Connection failed", "error");
      connected.value = false;
    } finally {
      connecting.value = false;
    }
  }

  async function connectWithRetry(maxRetries: number = MAX_RETRIES) {
    if (connecting.value) return;
    connecting.value = true;
    let cancelled = false;
    const cancelPromise = new Promise<void>((_, reject) => {
      cancelCurrentConnect = () => {
        cancelled = true;
        reject(new Error("cancelled"));
      };
    });
    try {
      for (let attempt = 1; attempt <= maxRetries; attempt++) {
        if (cancelled) break;
        try {
          await Promise.race([
            invoke("adb_connect_port", { ip: ipAddress.value, port: port.value }),
            cancelPromise,
          ]);
        } catch (connectError) {
          const msg = String(connectError);
          if (cancelled || msg.includes("cancelled")) {
            addLog("Connection cancelled", "info");
            toast.show("Connection cancelled", "info");
            try { await invoke("adb_disconnect"); } catch {}
            break;
          }
          addLog(`Connection attempt ${attempt}/${maxRetries} failed: ${msg}`, "error");
          if (attempt < maxRetries) {
            toast.show(`Retrying (${attempt}/${maxRetries})...`, "info");
            await Promise.race([
              new Promise(r => setTimeout(r, RETRY_DELAY_MS)),
              cancelPromise,
            ]);
            if (cancelled) {
              addLog("Connection cancelled", "info");
              toast.show("Connection cancelled", "info");
              try { await invoke("adb_disconnect"); } catch {}
              break;
            }
          } else {
            toast.show(`Connection failed after ${maxRetries} attempts`, "error");
            connected.value = false;
          }
          continue;
        }
        try {
          await onConnected(`${ipAddress.value}:${port.value}`, "manual");
        } catch (e) {
          addLog(`Post-connect setup failed: ${e}`, "error");
        }
        return;
      }
    } finally {
      connecting.value = false;
      cancelCurrentConnect = null;
    }
  }

  async function cancelConnect() {
    if (cancelCurrentConnect) {
      cancelCurrentConnect();
      cancelCurrentConnect = null;
    }
    connecting.value = false;
    try {
      await invoke("adb_cancel_connect");
    } catch (e) {
      addLog(`Cancel error: ${e}`, "error");
    }
  }

  async function autoConnect(): Promise<boolean> {
    autoConnectStatus.value = "detecting_usb";
    connecting.value = true;
    try {
      const devices = await invoke<{ id: string; state: string; transport: string; model: string }[]>("adb_devices");
      const usbDevices = devices.filter((d) => d.transport === "usb" && d.state === "device");
      const wifiDevices = devices.filter((d) => d.transport === "wifi" && d.state === "device");

      // Phase 1: USB device detected — enable TCP mode for seamless WiFi switch
      if (usbDevices.length > 0) {
        const usbDevice = usbDevices[0];
        transport.value = "usb";
        addLog(`USB device detected: ${usbDevice.model || usbDevice.id}`, "info");

        // Try to enable TCP mode
        autoConnectStatus.value = "enabling_tcp";
        try {
          await invoke("adb_tcpip", { serial: usbDevice.id, port: 5555 });
          addLog(`Wireless mode enabled on ${usbDevice.id} port 5555`, "success");
        } catch (e) {
          addLog(`Could not enable wireless mode: ${e}`, "error");
          // Fallback: connect via USB only
          connected.value = true;
          deviceId.value = usbDevice.id;
          try { await invoke("adb_set_device", { deviceId: usbDevice.id }); } catch {}
          pollFailCount = 0;
          addLog(`Connected to ${usbDevice.model || usbDevice.id} via USB`, "success");
          toast.show("Connected via USB", "success");
          addLog("Unplugging USB will disconnect the device", "info");
          try { await pollStats(); } catch {}
          startPolling();
          syncToggles();
          autoConnectStatus.value = "connected";
          return true;
        }

        // Detect IP address while USB is still connected
        autoConnectStatus.value = "detecting_ip";
        let deviceIp = "";
        try {
          deviceIp = await invoke<string>("adb_get_ip", { serial: usbDevice.id });
          addLog(`Detected IP: ${deviceIp}`, "success");
        } catch {
          addLog("Could not auto-detect IP address", "error");
        }

        // Connect via WiFi if IP is available
        if (deviceIp) {
          autoConnectStatus.value = "connecting_tcp";
          ipAddress.value = deviceIp;
          port.value = 5555;
          try {
            await invoke("adb_connect_port", { ip: deviceIp, port: 5555 });
            try { await onConnected(`${deviceIp}:5555`, "wifi"); } catch (e) {
              addLog(`Post-connect setup failed: ${e}`, "error");
            }
            addLog("You may unplug USB safely — device is connected via Wi-Fi", "info");
            return true;
          } catch (e) {
            addLog(`Wi-Fi connection failed: ${e}`, "error");
          }
        }

        // Fallback: connect via USB only
        connected.value = true;
        deviceId.value = usbDevice.id;
        try { await invoke("adb_set_device", { deviceId: usbDevice.id }); } catch {}
        pollFailCount = 0;
        addLog(`Connected to ${usbDevice.model || usbDevice.id} via USB`, "success");
        toast.show("Connected via USB", "success");
        addLog("Unplugging USB will disconnect the device", "info");
        try { await pollStats(); } catch {}
        startPolling();
        syncToggles();
        autoConnectStatus.value = "connected";
        return true;
      }

      // Phase 2: No USB — try existing WiFi device
      if (wifiDevices.length > 0) {
        const wifiDevice = wifiDevices[0];
        connected.value = true;
        deviceId.value = wifiDevice.id;
        transport.value = "wifi";
        try { await invoke("adb_set_device", { deviceId: wifiDevice.id }); } catch {}
        pollFailCount = 0;
        addLog(`Connected to ${wifiDevice.model || wifiDevice.id}`, "success");
        toast.show("Device connected", "success");
        try { await pollStats(); } catch {}
        startPolling();
        syncToggles();
        autoConnectStatus.value = "connected";
        return true;
      }

      // Phase 3: No devices found at all — try saved device
      const history = useConnectionHistoryStore();
      const last = history.getLastConnected();
      if (last) {
        addLog(`Trying saved device ${last.ip}:${last.port}...`, "info");
        try {
          await invoke("adb_connect_port", { ip: last.ip, port: last.port });
          try { await onConnected(`${last.ip}:${last.port}`, last.method); } catch (e) {
            addLog(`Post-connect setup failed: ${e}`, "error");
          }
          return true;
        } catch (e) {
          addLog(`Saved device unreachable: ${e}`, "error");
        }
      }

      if (devices.length > 0) {
        addLog(`${devices.length} device(s) found but none ready`, "error");
        toast.show("Device not ready", "error");
      } else {
        addLog("No devices found. Connect a USB device or enter an IP address.", "error");
        toast.show("No devices found", "error");
      }
      autoConnectStatus.value = "error";
      return false;
    } catch (e) {
      addLog(String(e), "error");
      autoConnectStatus.value = "error";
      return false;
    } finally {
      connecting.value = false;
    }
  }

  async function disconnect() {
    try {
      await invoke("adb_disconnect");
      connected.value = false;
      deviceId.value = "";
      isInitialLoad.value = true;
      stopPolling();
      addLog("Disconnected", "info");
      toast.show("Disconnected", "info");
      autoConnectStatus.value = "idle";
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function connectSaved(saved: { ip: string; port: number; method: "wifi" | "pairing" }) {
    ipAddress.value = saved.ip;
    port.value = saved.port;
    await connectWithPort(saved.ip, saved.port);
  }

  async function pairDevice(pairIp: string, pairPortVal: number, code: string) {
    connecting.value = true;
    try {
      await invoke("adb_pair", { ip: pairIp, pairPort: pairPortVal, code });
      addLog(`Paired with ${pairIp}:${pairPortVal}`, "success");
      toast.show("Pairing successful. Now connecting...", "success");
      port.value = 5555;
      await connectWithPort(pairIp, 5555);
    } catch (e) {
      addLog(String(e), "error");
      toast.show("Pairing failed", "error");
    } finally {
      connecting.value = false;
    }
  }

  async function handleDisconnect(reason: string) {
    connected.value = false;
    deviceId.value = "";
    resetStats();
    isInitialLoad.value = true;
    stopPolling();
    addLog(reason, "error");
    if (mirroring.value) {
      mirroring.value = false;
      try { await invoke("adb_stop_scrcpy"); } catch {}
    }
  }

  async function pollStats() {
    if (!connected.value || isPolling) return;
    const shouldShimmer = isInitialLoad.value;
    if (shouldShimmer) {
      isLoadingStats.value = true;
    }
    isPolling = true;
    try {
      const stats = await invoke<{
        battery: { level: number; status: string; health: string; temperature: number; plugged: boolean };
        cpu_usage: number;
        model: string;
        android_version: string;
        sdk_version: string;
      }>("adb_poll_device_stats");

      batteryLevel.value = stats.battery.level;
      batteryStatus.value = stats.battery.status;
      batteryHealth.value = stats.battery.health;
      batteryTemp.value = stats.battery.temperature;
      batteryPlugged.value = stats.battery.plugged;
      cpuUsage.value = stats.cpu_usage;
      model.value = stats.model || "—";
      androidVersion.value = stats.android_version || "—";
      sdkVersion.value = stats.sdk_version || "—";
      pollFailCount = 0;
      pollCount++;
      if (pollCount % 5 === 0) {
        syncToggles().catch(() => {});
      }
    } catch {
      pollFailCount++;
      addLog(`Device not responding (attempt ${pollFailCount}/3)`, "error");
      if (pollFailCount >= 3) {
        await handleDisconnect("Device disconnected unexpectedly");
        toast.show("Device disconnected unexpectedly", "error");
      }
    } finally {
      isInitialLoad.value = false;
      isPolling = false;
      isLoadingStats.value = false;
    }
  }

  function startPolling() {
    stopPolling();
    const settingsStore = useSettingsStore();
    const intervalMs = settingsStore.pollingInterval * 1000;
    async function scheduleNext() {
      await pollStats();
      if (connected.value) {
        pollTimeout = setTimeout(scheduleNext, intervalMs);
      }
    }
    pollTimeout = setTimeout(scheduleNext, intervalMs);
  }

  function stopPolling() {
    if (pollTimeout) {
      clearTimeout(pollTimeout);
      pollTimeout = null;
    }
  }

  async function executeCommand(cmd: string) {
    if (!cmd.trim()) return;
    addLog(`$ adb ${cmd}`, "info");
    try {
      const result = await invoke<string>("adb_custom_cmd", { command: cmd });
      if (result) addLog(result, "success");
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function toggleWifi(val: boolean) {
    try {
      await invoke("adb_set_wifi", { enable: val });
      wifiEnabled.value = val;
      addLog(`Wi-Fi ${val ? "enabled" : "disabled"}`, "success");
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function toggleData(val: boolean) {
    try {
      await invoke("adb_set_data", { enable: val });
      dataEnabled.value = val;
      addLog(`Data ${val ? "enabled" : "disabled"}`, "success");
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function toggleAirplane(val: boolean) {
    try {
      await invoke("adb_set_airplane", { enable: val });
      airplaneEnabled.value = val;
      addLog(`Airplane Mode ${val ? "enabled" : "disabled"}`, "success");
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function toggleBluetooth(val: boolean) {
    try {
      await invoke("adb_set_bluetooth", { enable: val });
      bluetoothEnabled.value = val;
      addLog(`Bluetooth ${val ? "enabled" : "disabled"}`, "success");
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function toggleShowTaps(val: boolean) {
    try {
      await invoke("adb_show_taps", { enable: val });
      showTapsEnabled.value = val;
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function toggleLayoutBounds(val: boolean) {
    try {
      await invoke("adb_layout_bounds", { enable: val });
      layoutBoundsEnabled.value = val;
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function syncToggles() {
    try {
      const toggles = await invoke<{
        wifi: boolean;
        data: boolean;
        airplane: boolean;
        bluetooth: boolean;
        show_taps: boolean;
        layout_bounds: boolean;
        stay_awake: boolean;
        brightness: number;
      }>("adb_sync_toggles");
      wifiEnabled.value = toggles.wifi;
      dataEnabled.value = toggles.data;
      airplaneEnabled.value = toggles.airplane;
      bluetoothEnabled.value = toggles.bluetooth;
      showTapsEnabled.value = toggles.show_taps;
      layoutBoundsEnabled.value = toggles.layout_bounds;
      stayAwakeEnabled.value = toggles.stay_awake;
      brightness.value = toggles.brightness;
    } catch {
      // Silently fail — toggles default to false
    }
  }

  async function toggleStayAwake(val: boolean) {
    try {
      await invoke("adb_stay_awake", { enable: val });
      stayAwakeEnabled.value = val;
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function pressHome() {
    try { await invoke("adb_key_home"); } catch (e) { addLog(String(e), "error"); }
  }

  async function pressBack() {
    try { await invoke("adb_key_back"); } catch (e) { addLog(String(e), "error"); }
  }

  async function pressRecent() {
    try { await invoke("adb_key_recent"); } catch (e) { addLog(String(e), "error"); }
  }

  async function rotateDevice() {
    try {
      const result = await invoke<string>("adb_rotate");
      addLog(result || "Rotated", "success");
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function pressVolUp() {
    try { await invoke("adb_key_vol_up"); addLog("Volume up", "success"); } catch (e) { addLog(String(e), "error"); }
  }

  async function pressVolDown() {
    try { await invoke("adb_key_vol_down"); addLog("Volume down", "success"); } catch (e) { addLog(String(e), "error"); }
  }

  async function pressMute() {
    try { await invoke("adb_key_mute"); addLog("Mute toggled", "success"); } catch (e) { addLog(String(e), "error"); }
  }

  async function pressPower() {
    try { await invoke("adb_key_power"); addLog("Power button pressed", "success"); } catch (e) { addLog(String(e), "error"); }
  }

  async function pressPrev() {
    try { await invoke("adb_key_prev"); addLog("Previous track", "success"); } catch (e) { addLog(String(e), "error"); }
  }

  async function pressPlayPause() {
    try { await invoke("adb_key_play_pause"); addLog("Play/Pause", "success"); } catch (e) { addLog(String(e), "error"); }
  }

  async function pressNext() {
    try { await invoke("adb_key_next"); addLog("Next track", "success"); } catch (e) { addLog(String(e), "error"); }
  }

  async function sendText() {
    const text = textInput.value.trim();
    if (!text) return;
    try {
      await invoke("adb_input_text", { text });
      addLog(`Sent text: "${text}"`, "success");
      textInput.value = "";
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function setBrightness(val: number) {
    const clamped = Math.max(0, Math.min(255, val));
    brightness.value = clamped;
    try {
      await invoke("adb_set_brightness", { value: clamped });
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function rebootRecovery() {
    const confirmed = await ask("Are you sure you want to reboot into recovery mode?", {
      title: "Reboot to Recovery",
      kind: "warning",
      okLabel: "Reboot",
      cancelLabel: "Cancel",
    });
    if (!confirmed) return;
    try {
      await invoke("adb_reboot_recovery");
      addLog("Rebooting to recovery...", "info");
      toast.show("Rebooting to recovery...", "info");
      connected.value = false;
      isInitialLoad.value = true;
      stopPolling();
    } catch (e) {
      addLog(String(e), "error");
    }
    showRebootMenu.value = false;
  }

  async function rebootBootloader() {
    const confirmed = await ask("Are you sure you want to reboot into bootloader?", {
      title: "Reboot to Bootloader",
      kind: "warning",
      okLabel: "Reboot",
      cancelLabel: "Cancel",
    });
    if (!confirmed) return;
    try {
      await invoke("adb_reboot_bootloader");
      addLog("Rebooting to bootloader...", "info");
      toast.show("Rebooting to bootloader...", "info");
      connected.value = false;
      isInitialLoad.value = true;
      stopPolling();
    } catch (e) {
      addLog(String(e), "error");
    }
    showRebootMenu.value = false;
  }

  async function takeScreenshot() {
    try {
      const settingsStore = useSettingsStore();
      const saveDir = settingsStore.screenshotSaveDir || null;
      const path = await invoke<string>("adb_screenshot", { saveDir });
      addLog(`Screenshot saved: ${path}`, "success");
      toast.show("Screenshot saved", "success");
      const lastSlash = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
      if (lastSlash >= 0) {
        const dir = path.substring(0, lastSlash);
        try { await invoke("open_folder", { path: dir }); } catch {}
      }
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function rebootDevice() {
    const confirmed = await ask("Are you sure you want to reboot the device?", {
      title: "Reboot Device",
      kind: "warning",
      okLabel: "Reboot",
      cancelLabel: "Cancel",
    });
    if (!confirmed) return;
    try {
      await invoke("adb_reboot");
      addLog("Rebooting device...", "info");
      toast.show("Rebooting device...", "info");
      connected.value = false;
      isInitialLoad.value = true;
      stopPolling();
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function checkScrcpy() {
    try {
      const status = await invoke<{ available: boolean; path: string | null }>("adb_find_scrcpy");
      scrcpyAvailable.value = status.available;
      scrcpyPath.value = status.path;
    } catch {
      scrcpyAvailable.value = false;
    }
  }

  async function launchMirror() {
    try {
      const settingsStore = useSettingsStore();
      const args = settingsStore.getScrcpyArgs();
      const bitrate = args.includes("--video-bit-rate")
        ? args[args.indexOf("--video-bit-rate") + 1]
        : null;
      const maxSize = args.includes("--max-size")
        ? args[args.indexOf("--max-size") + 1]
        : null;
      const saveDir = settingsStore.recordingSaveDir || null;
      const recordingFormat = settingsStore.recordingFormat;
      await invoke("adb_launch_scrcpy", {
        record: recordingScreen.value,
        saveDir,
        videoBitrate: bitrate,
        maxSize,
        recordingFormat,
      });
      mirroring.value = true;
      addLog("scrcpy launched", "success");
      toast.show("Mirroring started", "success");
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  async function stopMirror() {
    try {
      await invoke("adb_stop_scrcpy");
      mirroring.value = false;
      addLog("scrcpy stopped", "info");
    } catch (e) {
      addLog(String(e), "error");
    }
  }

  return {
    connected, connecting, ipAddress, port, deviceId, connectMethod, transport, autoConnectStatus,
    isLoadingStats, isInitialLoad,
    batteryLevel, batteryStatus, batteryHealth, batteryTemp, batteryPlugged, batteryColor,
    cpuUsage, model, androidVersion, sdkVersion,
    logs, commandInput,
    scrcpyAvailable, scrcpyPath, mirroring, recordingScreen,
    wifiEnabled, dataEnabled, airplaneEnabled, bluetoothEnabled,
    showTapsEnabled, layoutBoundsEnabled, stayAwakeEnabled,
    brightness, textInput, showRebootMenu,
    addLog, clearLogs, connectWithPort, connectWithRetry, cancelConnect, autoConnect, disconnect,
    onConnected, handleDisconnect, pollStats, startPolling, executeCommand,
    connectSaved, pairDevice,
    toggleWifi, toggleData, toggleAirplane, toggleBluetooth, toggleShowTaps,
    syncToggles, toggleLayoutBounds, toggleStayAwake,
    pressHome, pressBack, pressRecent, rotateDevice,
    pressVolUp, pressVolDown, pressMute, pressPower,
    pressPrev, pressPlayPause, pressNext,
    sendText, setBrightness,
    rebootDevice, rebootRecovery, rebootBootloader,
    takeScreenshot,
    checkScrcpy, launchMirror, stopMirror,
  };
});