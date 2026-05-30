import { ref, onBeforeUnmount } from 'vue';
import { Channel } from '@tauri-apps/api/core';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useLogcatStore } from '../stores/logcat';
import type { LogEntry } from '../stores/logcat';

const RESTART_DELAY_MS = 1000;
const MAX_START_RETRIES = 3;
const RETRY_BACKOFF_MS = 1000;

export function useLogcatChannel() {
  const store = useLogcatStore();

  // Keep Channel as a plain variable — NEVER wrap in a Vue reactive ref.
  // Tauri Channel uses private fields (#). Vue's Proxy breaks assignment
  // to those fields (e.g. channel.value.onmessage = ...) and throws:
  //   TypeError: Cannot write private member to an object whose class did not declare it
  let channel: Channel<LogEntry> | null = null;

  const stagingQueue = ref<LogEntry[]>([]);
  let flushInterval: ReturnType<typeof setInterval> | null = null;
  let isListening = false;
  let eventUnlisten: UnlistenFn | null = null;

  /**
   * Start the logcat stream. Returns `true` on success, `false` on failure.
   */
  async function start(): Promise<boolean> {
    if (isListening) return true;

    const ch = new Channel<LogEntry>();
    ch.onmessage = (entry) => {
      stagingQueue.value.push(entry);
    };

    channel = ch;
    isListening = true;
    store.streaming = true;
    store.paused = false;
    store.status = 'LIVE';
    store.error = '';

    flushInterval = setInterval(() => {
      if (stagingQueue.value.length === 0) return;
      if (store.paused) {
        stagingQueue.value = [];
        return;
      }
      const batch = stagingQueue.value.splice(0);
      store.appendEntries(batch);
    }, 100);

    // Listen for backend process-exit events so we know when logcat dies
    if (eventUnlisten) {
      eventUnlisten();
      eventUnlisten = null;
    }
    eventUnlisten = await listen<{ cancelled: boolean; exitCode: number | null; stderr: string }>(
      'logcat-exited',
      (event) => {
        const { cancelled, exitCode, stderr } = event.payload;

        // If the stop was user-initiated (Refresh/Stop), the restart flow
        // already handles state – ignore this event.
        if (cancelled) return;

        // Unexpected process exit – update frontend state so the user knows
        if (isListening) {
          cleanup();
          store.streaming = false;
          const errMsg = stderr
            ? `Logcat process exited (code ${exitCode ?? 'unknown'}): ${stderr}`
            : `Logcat process exited unexpectedly (code ${exitCode ?? 'unknown'})`;
          store.error = errMsg;
          store.status = 'DISCONNECTED';
        }
      }
    );

    try {
      await invoke('adb_start_logcat', { channel: ch });
      return true;
    } catch (e) {
      store.streaming = false;
      store.status = 'ERROR';
      store.error = String(e);
      cleanup();
      return false;
    }
  }

  async function stop() {
    if (!isListening) return;
    cleanup();
    try {
      await invoke('adb_stop_logcat');
    } catch (e) {
      console.error('adb_stop_logcat error:', e);
    }
    store.streaming = false;
    if (store.status !== 'ERROR' && store.status !== 'DISCONNECTED') {
      store.status = 'IDLE';
    }
  }

  async function restart() {
    // Always try to stop any existing backend process
    try {
      await invoke('adb_stop_logcat');
    } catch {
      // Ignore — the process might not be running
    }

    // Clean up local channel state
    cleanup();

    // Give the ADB daemon time to fully tear down the old connection.
    // Without this delay a new `adb logcat` may fail or exit immediately
    // because the daemon is still cleaning up the previous session.
    await new Promise<void>((r) => setTimeout(r, RESTART_DELAY_MS));

    // Reset store state for a fresh start (keep existing log entries)
    store.streaming = false;
    store.error = '';

    // Attempt to start with retries — the ADB daemon might still need a moment
    let lastError: string | null = null;
    for (let attempt = 1; attempt <= MAX_START_RETRIES; attempt++) {
      const ok = await start();
      if (ok) {
        return; // success
      }
      lastError = store.error;
      if (attempt < MAX_START_RETRIES) {
        // The previous start() call set status to ERROR and called cleanup(),
        // so we can safely try again after a short back-off.
        const backoff = RETRY_BACKOFF_MS * attempt;
        await new Promise<void>((r) => setTimeout(r, backoff));
      }
    }
    // All retries exhausted
    store.status = 'ERROR';
    store.error = lastError ?? 'Failed to restart logcat after multiple attempts';
  }

  function cleanup() {
    isListening = false;
    if (flushInterval) {
      clearInterval(flushInterval);
      flushInterval = null;
    }
    stagingQueue.value = [];
    if (channel) {
      channel.onmessage = () => {};
      channel = null;
    }
    if (eventUnlisten) {
      eventUnlisten();
      eventUnlisten = null;
    }
  }

  onBeforeUnmount(() => {
    stop();
  });

  return { start, stop, restart, cleanup };
}
