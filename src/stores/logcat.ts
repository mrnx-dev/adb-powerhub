import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export interface LogEntry {
  id: number;
  timestamp: string;
  pid: string;
  tid: string;
  level: 'V' | 'D' | 'I' | 'W' | 'E' | 'F';
  tag: string;
  message: string;
  raw?: string;
  firstLine: string;
  contLines: string[];
  hasCont: boolean;
  displayMessage: string;
}

export type LogcatStatus = 'IDLE' | 'LIVE' | 'PAUSED' | 'DISCONNECTED' | 'ERROR';

const MAX_BUFFER = 500;
const MAX_LINE_LEN = 500;

export const useLogcatStore = defineStore('logcat', () => {
  const entries = ref<LogEntry[]>([]);
  const lastAppendAt = ref(0); // monotonic tick, increments on every append
  const droppedCount = ref(0);
  const nextId = ref(0);

  const filterLevel = ref<'ALL' | 'V' | 'D' | 'I' | 'W' | 'E' | 'F'>('ALL');
  const tagQuery = ref('');
  const searchQuery = ref('');

  const streaming = ref(false);
  const paused = ref(false);
  const status = ref<LogcatStatus>('IDLE');
  const error = ref('');
  const autoScroll = ref(true);

  const startRequested = ref(false);
  const restartRequested = ref(false);
  const restarting = ref(false);

  const filteredEntries = computed(() => {
    let result = entries.value;
    if (filterLevel.value !== 'ALL') {
      result = result.filter((e) => e.level === filterLevel.value);
    }
    if (tagQuery.value.trim()) {
      const q = tagQuery.value.toLowerCase();
      result = result.filter((e) => e.tag.toLowerCase().includes(q));
    }
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter((e) => {
        const haystack = `${e.timestamp} ${e.pid} ${e.tid} ${e.level} ${e.tag} ${e.message}`;
        return haystack.toLowerCase().includes(q);
      });
    }
    return result;
  });

  const visibleCount = computed(() => filteredEntries.value.length);
  const totalCount = computed(() => entries.value.length);

  function appendEntries(newEntries: LogEntry[]) {
    for (const entry of newEntries) {
      if (entry.id > nextId.value + 1 && nextId.value !== 0) {
        droppedCount.value += entry.id - nextId.value - 1;
      }
      nextId.value = entry.id;

      if (!entry.firstLine) {
        const lines = entry.message.split('\n');
        entry.firstLine = lines[0] ?? '';
        entry.contLines = lines.slice(1);
        entry.hasCont = entry.contLines.length > 0;
      }
      if (!entry.displayMessage) {
        entry.displayMessage =
          entry.firstLine.length > MAX_LINE_LEN
            ? entry.firstLine.slice(0, MAX_LINE_LEN) + '…'
            : entry.firstLine;
      }
    }

    const combined = [...entries.value, ...newEntries];
    if (combined.length > MAX_BUFFER) {
      entries.value = combined.slice(combined.length - MAX_BUFFER);
    } else {
      entries.value = combined;
    }

    lastAppendAt.value++;
  }

  function appendEntry(entry: LogEntry) {
    appendEntries([entry]);
  }

  function clearLocalBuffer() {
    entries.value = [];
    nextId.value = 0;
    droppedCount.value = 0;
    lastAppendAt.value++;
    // Preserve status: only reset to IDLE if not streaming
    if (!streaming.value) {
      status.value = 'IDLE';
    }
  }

  async function exportLogs() {
    if (filteredEntries.value.length === 0) return;
    const content = filteredEntries.value
      .map((e) => {
        if (e.raw) return e.raw;
        const header = `${e.timestamp}  ${e.pid}  ${e.tid}  ${e.level}  ${e.tag}: ${e.firstLine}`;
        const cont = e.contLines.map((l) => `      ${l}`).join('\n');
        return cont ? `${header}\n${cont}` : header;
      })
      .join('\n');

    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const path = await save({
        filters: [{ name: 'Log', extensions: ['log', 'txt'] }],
        defaultPath: `logcat_${new Date().toISOString().slice(0, 19).replace(/[:.]/g, '-')}.log`,
      });
      if (!path) return;
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('write_text_file', { path, content });
    } catch (e) {
      error.value = String(e);
    }
  }

  function setPaused(val: boolean) {
    paused.value = val;
    status.value = val ? 'PAUSED' : streaming.value ? 'LIVE' : 'IDLE';
  }

  function requestStart() {
    startRequested.value = true;
  }

  function requestRestart() {
    if (restarting.value) return;
    restartRequested.value = true;
  }

  return {
    entries,
    filteredEntries,
    visibleCount,
    totalCount,
    filterLevel,
    tagQuery,
    searchQuery,
    streaming,
    paused,
    status,
    error,
    autoScroll,
    droppedCount,
    lastAppendAt,
    startRequested,
    restartRequested,
    restarting,
    appendEntries,
    appendEntry,
    clearLocalBuffer,
    exportLogs,
    setPaused,
    requestStart,
    requestRestart,
  };
});
