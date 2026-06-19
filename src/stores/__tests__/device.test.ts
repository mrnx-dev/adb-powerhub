import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Per-command mock for Tauri invoke — the module boundary IS the test seam
// (Blueprint §4.2 deviation: no production runAdbCommand injection needed;
// mocking at the import boundary gives the same coverage with zero prod touch).
const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import { useDeviceStore } from '../device';
import { useSettingsStore } from '../settings';

const STATS = {
  battery: {
    level: 50,
    status: 'Discharging',
    health: 'Good',
    temperature: 30,
    plugged: false,
    voltage: 4000,
  },
  cpu_usage: 10,
  model: 'Pixel Test',
  android_version: '14',
  sdk_version: '34',
  ram_total_mb: 8000,
  ram_available_mb: 4000,
  storage_total_gb: 128,
  storage_used_gb: 64,
  screen_width: 1080,
  screen_height: 2400,
};

const TOGGLES = {
  wifi: false,
  data: false,
  airplane: false,
  bluetooth: false,
  show_taps: false,
  layout_bounds: false,
  stay_awake: false,
  brightness: 128,
  density: 420,
  density_override: null,
  density_physical: 420,
};

/** Default handler: resolves known commands, rejects unknown (most callers catch). */
function defaultHandler(cmd: string) {
  switch (cmd) {
    case 'adb_connect_port':
      return 'OK';
    case 'adb_poll_device_stats':
      return STATS;
    case 'adb_sync_toggles':
      return TOGGLES;
    case 'adb_disconnect':
      return null;
    default:
      return Promise.reject(new Error(`unmocked command: ${cmd}`));
  }
}

describe('useDeviceStore — connection state machine', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    invokeMock.mockReset();
    invokeMock.mockImplementation((cmd: string) => Promise.resolve(defaultHandler(cmd)));
    // Disable auto-reconnect so handleDisconnect doesn't schedule a watcher.
    useSettingsStore().autoReconnect = false;
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('connectWithPort transitions connecting → connected on success', async () => {
    const store = useDeviceStore();
    store.ipAddress = '192.168.1.50';
    store.port = 5555;
    expect(store.connected).toBe(false);
    expect(store.connecting).toBe(false);

    await store.connectWithPort('192.168.1.50', 5555);

    expect(store.connected).toBe(true);
    expect(store.connecting).toBe(false); // reset in finally
    expect(store.deviceId).toBe('192.168.1.50:5555');
    expect(store.model).toBe('Pixel Test'); // populated by pollStats
    // adb_connect_port + adb_poll_device_stats + adb_sync_toggles were called
    const calls = invokeMock.mock.calls.map((c) => c[0]);
    expect(calls).toContain('adb_connect_port');
    expect(calls).toContain('adb_poll_device_stats');
  });

  it('connectWithPort keeps disconnected on adb_connect_port failure', async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'adb_connect_port') return Promise.reject(new Error('timeout'));
      return Promise.resolve(defaultHandler(cmd));
    });

    const store = useDeviceStore();
    await store.connectWithPort('10.0.0.99', 5555);

    expect(store.connected).toBe(false);
    expect(store.connecting).toBe(false);
  });

  it('auto-disconnects after 3 consecutive pollStats failures', async () => {
    const store = useDeviceStore();
    // Simulate an already-connected device.
    store.connected = true;
    expect(store.pollStats).toBeDefined();

    // adb_poll_device_stats rejects on every call.
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'adb_poll_device_stats') return Promise.reject(new Error('device gone'));
      return Promise.resolve(defaultHandler(cmd));
    });

    // First two failures increment the counter but stay connected.
    await store.pollStats();
    expect(store.connected).toBe(true);
    await store.pollStats();
    expect(store.connected).toBe(true);

    // Third failure triggers handleDisconnect → connected flips to false.
    await store.pollStats();
    expect(store.connected).toBe(false);
  });

  it('a successful poll resets the failure counter (no false disconnect)', async () => {
    const store = useDeviceStore();
    store.connected = true;

    let fail = true;
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'adb_poll_device_stats' && fail) {
        return Promise.reject(new Error('transient'));
      }
      return Promise.resolve(defaultHandler(cmd));
    });

    await store.pollStats(); // fail #1
    expect(store.connected).toBe(true);
    fail = false;
    await store.pollStats(); // success → counter resets
    expect(store.connected).toBe(true);
    fail = true;
    await store.pollStats(); // fail #1 again (counter was reset)
    await store.pollStats(); // fail #2
    expect(store.connected).toBe(true); // only 2 consecutive fails, not 3
  });
});
