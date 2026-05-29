import { defineStore } from 'pinia';
import { ref } from 'vue';
import { load, Store } from '@tauri-apps/plugin-store';

export interface SavedDevice {
  id: string;
  ip: string;
  port: number;
  label: string;
  model: string;
  lastConnected: string;
  method: 'wifi' | 'pairing';
}

function deviceId(ip: string, port: number): string {
  return `${ip}:${port}`;
}

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load('connection_history.json', { autoSave: true, defaults: {} });
  }
  return storeInstance;
}

export const useConnectionHistoryStore = defineStore('connectionHistory', () => {
  const devices = ref<SavedDevice[]>([]);

  async function init() {
    try {
      const s = await getStore();
      const saved = await s.get<SavedDevice[]>('devices');
      if (saved) {
        devices.value = saved;
      }
    } catch {
      // first run
    }
  }

  async function persist() {
    try {
      const s = await getStore();
      await s.set('devices', devices.value);
    } catch {
      // ignore
    }
  }

  function save(device: SavedDevice) {
    const existing = devices.value.find((d) => d.id === device.id);
    if (existing) {
      existing.lastConnected = device.lastConnected;
      existing.label = device.label || existing.label;
      existing.model = device.model || existing.model;
      existing.method = device.method;
    } else {
      devices.value.unshift(device);
    }
    persist();
  }

  function remove(id: string) {
    devices.value = devices.value.filter((d) => d.id !== id);
    persist();
  }

  function getByIp(ip: string, port: number): SavedDevice | undefined {
    return devices.value.find((d) => d.ip === ip && d.port === port);
  }

  function getLastConnected(): SavedDevice | undefined {
    if (devices.value.length === 0) return undefined;
    return devices.value.reduce((a, b) =>
      new Date(a.lastConnected) > new Date(b.lastConnected) ? a : b
    );
  }

  function getAll(): SavedDevice[] {
    return [...devices.value].sort(
      (a, b) => new Date(b.lastConnected).getTime() - new Date(a.lastConnected).getTime()
    );
  }

  return {
    devices,
    init,
    save,
    remove,
    getByIp,
    getLastConnected,
    getAll,
    deviceId,
  };
});
