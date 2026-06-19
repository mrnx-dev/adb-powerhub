import { describe, it, expect, vi, beforeEach } from 'vitest';

// In-memory fake Tauri store so presets logic can run without Tauri runtime.
const fakeData = new Map<string, unknown>();
const fakeStore = {
  get: vi.fn((key: string) => Promise.resolve(fakeData.get(key))),
  set: vi.fn((key: string, value: unknown) => {
    fakeData.set(key, value);
    return Promise.resolve();
  }),
  save: vi.fn(() => Promise.resolve()),
};

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(() => Promise.resolve(fakeStore)),
}));

import { usePresetsStore } from '../presets';

describe('usePresetsStore', () => {
  beforeEach(() => {
    fakeData.clear();
    fakeStore.get.mockClear();
    fakeStore.set.mockClear();
  });

  it('addPreset validates name + command and rejects empty fields', async () => {
    const store = usePresetsStore();
    await store.loadPresets(); // seed built-ins so the store is initialized

    const nameErr = await store.addPreset('   ', 'shell pm list packages');
    expect(nameErr).toMatch(/name is required/i);

    const cmdErr = await store.addPreset('MyPreset', '   ');
    expect(cmdErr).toMatch(/command is required/i);
  });

  it('addPreset strips an "adb " prefix from the command', async () => {
    const store = usePresetsStore();
    await store.loadPresets();

    const err = await store.addPreset('ListPkgs', 'adb shell pm list packages -3');
    expect(err).toBeNull();
    const added = store.presets.find((p) => p.name === 'ListPkgs');
    expect(added?.command).toBe('shell pm list packages -3');
  });

  it('addPreset rejects duplicate names (case-insensitive)', async () => {
    const store = usePresetsStore();
    await store.loadPresets();
    await store.addPreset('Snapshot', 'shell screencap -p');

    const dup = await store.addPreset('SNAPSHOT', 'shell something');
    expect(dup).toMatch(/already exists/i);
  });

  it('deletePreset removes the preset by id and keeps order', async () => {
    const store = usePresetsStore();
    await store.loadPresets();
    // Clear built-ins so we reason about a known set.
    store.presets = [];
    await store.addPreset('First', 'shell echo 1');
    await store.addPreset('Second', 'shell echo 2');
    await store.addPreset('Third', 'shell echo 3');

    const second = store.presets.find((p) => p.name === 'Second')!;
    await store.deletePreset(second.id);

    expect(store.presets.map((p) => p.name)).toEqual(['First', 'Third']);
  });

  it('updatePreset edits an existing preset and rejects unknown id', async () => {
    const store = usePresetsStore();
    await store.loadPresets();
    await store.addPreset('RenameMe', 'shell echo 1');

    const target = store.presets.find((p) => p.name === 'RenameMe')!;
    const err = await store.updatePreset(target.id, 'Renamed', 'shell echo 2');
    expect(err).toBeNull();
    expect(store.presets.find((p) => p.id === target.id)?.name).toBe('Renamed');

    const missing = await store.updatePreset('does-not-exist', 'X', 'shell y');
    expect(missing).toMatch(/not found/i);
  });

  it('persist shape: saved presets array contains {id,name,command}', async () => {
    const store = usePresetsStore();
    await store.loadPresets();
    await store.addPreset('Persisted', 'shell echo 1');

    const saved = fakeData.get('presets') as Array<{ id: string; name: string; command: string }>;
    expect(saved).toBeDefined();
    const entry = saved.find((p) => p.name === 'Persisted');
    expect(entry).toBeDefined();
    expect(typeof entry?.id).toBe('string');
    expect(entry?.command).toBe('shell echo 1');
  });
});
