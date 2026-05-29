import { defineStore } from 'pinia';
import { ref } from 'vue';
import { load, type Store } from '@tauri-apps/plugin-store';
import { useToastStore } from './toast';
import { useDeviceStore } from './device';

let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
  if (!storeInstance) {
    storeInstance = await load('presets.json', { autoSave: true, defaults: {} });
  }
  return storeInstance;
}

export interface Preset {
  id: string;
  name: string;
  command: string;
}

const BUILT_INS: Preset[] = [
  { id: 'builtin-list-packages', name: 'List Packages', command: 'shell pm list packages -3' },
  { id: 'builtin-battery-info', name: 'Battery Info', command: 'shell dumpsys battery' },
  {
    id: 'builtin-device-info',
    name: 'Device Info',
    command: 'shell getprop ro.product.model ro.build.version.release',
  },
  { id: 'builtin-network-config', name: 'Network Config', command: 'shell ip addr show wlan0' },
  { id: 'builtin-running-processes', name: 'Running Processes', command: 'shell ps -A | head -20' },
];

export const usePresetsStore = defineStore('presets', () => {
  const toast = useToastStore();
  const presets = ref<Preset[]>([]);
  const isLoading = ref(false);

  async function loadPresets() {
    try {
      isLoading.value = true;
      const s = await getStore();
      const saved = await s.get<Preset[]>('presets');
      const initialized = await s.get<boolean>('builtins_initialized');

      if (saved && saved.length > 0) {
        presets.value = saved;
      }

      if (!initialized) {
        presets.value = [...BUILT_INS];
        await s.set('presets', presets.value);
        await s.set('builtins_initialized', true);
      }
    } catch {
      toast.show('Failed to load presets', 'error');
    } finally {
      isLoading.value = false;
    }
  }

  async function savePresets() {
    try {
      const s = await getStore();
      await s.set('presets', presets.value);
    } catch {
      toast.show('Failed to save presets', 'error');
    }
  }

  function validate(name: string, command: string, excludeId?: string): string | null {
    const n = name.trim();
    const c = command.trim();
    if (!n) return 'Name is required';
    if (n.length > 50) return 'Name must be 50 characters or less';
    if (!c) return 'Command is required';
    if (c.length > 500) return 'Command must be 500 characters or less';
    const normalized = n.toLowerCase();
    const exists = presets.value.some(
      (p) => p.name.toLowerCase() === normalized && p.id !== excludeId
    );
    if (exists) return `Preset "${n}" already exists`;
    return null;
  }

  function stripAdbPrefix(command: string): string {
    return command.replace(/^adb\s+/, '');
  }

  async function addPreset(name: string, command: string): Promise<string | null> {
    const clean = stripAdbPrefix(command);
    const error = validate(name, clean);
    if (error) return error;

    const preset: Preset = {
      id: crypto.randomUUID(),
      name: name.trim(),
      command: clean,
    };

    presets.value.push(preset);
    await savePresets();
    toast.show(`Preset "${preset.name}" saved`, 'success');
    return null;
  }

  async function updatePreset(id: string, name: string, command: string): Promise<string | null> {
    const clean = stripAdbPrefix(command);
    const error = validate(name, clean, id);
    if (error) return error;

    const idx = presets.value.findIndex((p) => p.id === id);
    if (idx === -1) return 'Preset not found';

    presets.value[idx] = { id, name: name.trim(), command: clean };
    await savePresets();
    toast.show(`Preset "${name.trim()}" updated`, 'success');
    return null;
  }

  async function deletePreset(id: string) {
    const preset = presets.value.find((p) => p.id === id);
    if (!preset) return;

    presets.value = presets.value.filter((p) => p.id !== id);
    await savePresets();
    toast.show(`Preset "${preset.name}" deleted`, 'info');
  }

  function getPresetByIndex(index: number): Preset | undefined {
    return presets.value[index];
  }

  async function runPreset(preset: Preset) {
    const deviceStore = useDeviceStore();
    if (!deviceStore.connected) {
      toast.show('Connect a device to run presets', 'info');
      return;
    }
    deviceStore.addLog(`[Preset] ${preset.name}: ${preset.command}`, 'info');
    await deviceStore.executeCommand(preset.command);
  }

  return {
    presets,
    isLoading,
    loadPresets,
    addPreset,
    updatePreset,
    deletePreset,
    getPresetByIndex,
    runPreset,
  };
});
