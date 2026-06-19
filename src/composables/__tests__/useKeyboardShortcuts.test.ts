import { describe, it, expect, beforeEach } from 'vitest';
import { defineComponent, h } from 'vue';
import { mount } from '@vue/test-utils';
import { useKeyboardShortcuts } from '../useKeyboardShortcuts';
import { useNavigationStore } from '../../stores/navigation';

function mountWithShortcuts() {
  const Wrapper = defineComponent({
    setup() {
      useKeyboardShortcuts();
      return () => h('div');
    },
  });
  return mount(Wrapper);
}

function keydown(
  key: string,
  opts: { ctrlKey?: boolean; shiftKey?: boolean; altKey?: boolean; metaKey?: boolean } = {}
) {
  const ev = new KeyboardEvent('keydown', {
    key,
    bubbles: true,
    ctrlKey: !!opts.ctrlKey,
    shiftKey: !!opts.shiftKey,
    altKey: !!opts.altKey,
    metaKey: !!opts.metaKey,
  });
  Object.defineProperty(ev, 'target', { value: document.body });
  document.dispatchEvent(ev);
  return ev;
}

describe('useKeyboardShortcuts', () => {
  let navStore: ReturnType<typeof useNavigationStore>;

  beforeEach(() => {
    navStore = useNavigationStore();
    navStore.closeConnectPanel();
    navStore.navigateTo('dashboard');
  });

  it('Escape closes the connect panel when it is open', () => {
    mountWithShortcuts();
    navStore.openConnectPanel();
    expect(navStore.connectPanelOpen).toBe(true);

    keydown('Escape');
    expect(navStore.connectPanelOpen).toBe(false);
  });

  it('Escape does nothing to the panel when it is already closed', () => {
    mountWithShortcuts();
    expect(navStore.connectPanelOpen).toBe(false);
    keydown('Escape');
    expect(navStore.connectPanelOpen).toBe(false);
  });

  it('Ctrl+K navigates to the dashboard', () => {
    mountWithShortcuts();
    navStore.navigateTo('apps'); // start elsewhere
    expect(navStore.currentPage).toBe('apps');

    keydown('k', { ctrlKey: true });
    expect(navStore.currentPage).toBe('dashboard');
  });

  it('Ctrl+Shift+S navigates to screenshots', () => {
    mountWithShortcuts();
    keydown('S', { ctrlKey: true, shiftKey: true });
    expect(navStore.currentPage).toBe('screenshots');
  });

  it('Ctrl+, navigates to settings', () => {
    mountWithShortcuts();
    keydown(',', { ctrlKey: true });
    expect(navStore.currentPage).toBe('settings');
  });

  it('listener is removed on unmount — shortcuts stop firing', () => {
    const wrapper = mountWithShortcuts();
    wrapper.unmount();

    navStore.openConnectPanel();
    keydown('Escape');
    // No handler → panel stays open.
    expect(navStore.connectPanelOpen).toBe(true);
  });
});
