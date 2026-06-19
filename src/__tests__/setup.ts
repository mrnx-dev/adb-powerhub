/**
 * Vitest global setup.
 *
 * Runs before every test file. Activates a fresh Pinia instance so stores
 * can be used without a Vue app mount, and resets DOM state between tests
 * (happy-dom).
 */
import { beforeEach, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';

beforeEach(() => {
  // Fresh Pinia per test → isolated store state (PRD NFR-2: deterministic).
  setActivePinia(createPinia());

  // Reset happy-dom document between tests so leftover DOM doesn't leak.
  document.body.innerHTML = '';
});

// Tauri APIs are not available in the happy-dom test environment. Provide a
// safe default mock so any code path that touches `invoke`/event APIs in a
// unit test fails loudly instead of silently throwing at import time.
// Individual tests override these via `vi.mock(...)` where needed.
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockRejectedValue(new Error('invoke not mocked in this test')),
}));
