import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { defineComponent, h, ref } from 'vue';
import { mount, flushPromises } from '@vue/test-utils';
import { useLongPress } from '../useLongPress';

// Mount a host component, then point the composable at a real div attached to
// the document. We set the target ref AFTER mount + flush microtasks so the
// composable's watch fires synchronously and attaches its listeners before we
// dispatch events.
async function makeWrapper(callback: () => void, duration = 500) {
  const target = ref<HTMLElement | null>(null);
  let pressed = { value: false };
  const Wrapper = defineComponent({
    setup() {
      const lp = useLongPress(target, callback, { duration });
      pressed = lp.isPressed;
      return () => h('div');
    },
  });
  mount(Wrapper);

  const div = document.createElement('div');
  document.body.appendChild(div);
  target.value = div;
  await flushPromises(); // let the watch(…, {immediate}) callback run + attach

  return {
    el: () => div,
    getPressed: () => pressed.value,
    cleanup: () => div.remove(),
  };
}

function pointer(type: string, x = 0, y = 0) {
  return new PointerEvent(type, { clientX: x, clientY: y, bubbles: true });
}

describe('useLongPress', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it('fires the callback after the duration and sets isPressed', async () => {
    const cb = vi.fn();
    const { el, getPressed, cleanup } = await makeWrapper(cb, 500);

    el().dispatchEvent(pointer('pointerdown'));
    expect(cb).not.toHaveBeenCalled();

    vi.advanceTimersByTime(499);
    expect(cb).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1); // total 500
    expect(cb).toHaveBeenCalledTimes(1);
    expect(getPressed()).toBe(true);
    cleanup();
  });

  it('does NOT fire if pointerup arrives before the duration', async () => {
    const cb = vi.fn();
    const { el, getPressed, cleanup } = await makeWrapper(cb, 500);

    el().dispatchEvent(pointer('pointerdown'));
    vi.advanceTimersByTime(200);
    el().dispatchEvent(pointer('pointerup'));
    vi.advanceTimersByTime(500); // well past the duration

    expect(cb).not.toHaveBeenCalled();
    expect(getPressed()).toBe(false);
    cleanup();
  });

  it('cancels when the pointer moves beyond the move threshold', async () => {
    const cb = vi.fn();
    const { el, cleanup } = await makeWrapper(cb, 500);

    el().dispatchEvent(pointer('pointerdown', 0, 0));
    el().dispatchEvent(pointer('pointermove', 50, 0)); // > 10px
    vi.advanceTimersByTime(500);

    expect(cb).not.toHaveBeenCalled();
    cleanup();
  });

  it('stays firing when the pointer moves within the threshold', async () => {
    const cb = vi.fn();
    const { el, cleanup } = await makeWrapper(cb, 500);

    el().dispatchEvent(pointer('pointerdown', 0, 0));
    el().dispatchEvent(pointer('pointermove', 5, 5)); // within 10px
    vi.advanceTimersByTime(500);

    expect(cb).toHaveBeenCalledTimes(1);
    cleanup();
  });

  it('pointercancel cancels the pending press', async () => {
    const cb = vi.fn();
    const { el, cleanup } = await makeWrapper(cb, 500);

    el().dispatchEvent(pointer('pointerdown'));
    el().dispatchEvent(new PointerEvent('pointercancel'));
    vi.advanceTimersByTime(500);

    expect(cb).not.toHaveBeenCalled();
    cleanup();
  });
});
