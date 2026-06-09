import { ref, watch, onUnmounted, type Ref } from 'vue';

export function useLongPress(
  target: Ref<HTMLElement | null>,
  callback: () => void,
  options?: { duration?: number }
) {
  const isPressed = ref(false);
  const duration = options?.duration ?? 500;
  const MOVE_THRESHOLD = 10;

  let timer: ReturnType<typeof setTimeout> | null = null;
  let startX = 0;
  let startY = 0;
  let fired = false;

  function onPointerDown(e: PointerEvent) {
    fired = false;
    startX = e.clientX;
    startY = e.clientY;
    isPressed.value = false;

    timer = setTimeout(() => {
      fired = true;
      isPressed.value = true;
      callback();
    }, duration);
  }

  function onPointerMove(e: PointerEvent) {
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;
    if (Math.abs(dx) > MOVE_THRESHOLD || Math.abs(dy) > MOVE_THRESHOLD) {
      cancel();
    }
  }

  function onPointerUp() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    if (fired) {
      isPressed.value = false;
    }
  }

  function cancel() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    isPressed.value = false;
  }

  let currentEl: HTMLElement | null = null;

  function attachTo(el: HTMLElement | null) {
    // Detach from previous element
    if (currentEl) {
      currentEl.removeEventListener('pointerdown', onPointerDown as EventListener);
      currentEl.removeEventListener('pointermove', onPointerMove as EventListener);
      currentEl.removeEventListener('pointerup', onPointerUp as EventListener);
      currentEl.removeEventListener('pointercancel', cancel as EventListener);
    }
    currentEl = el;
    if (el) {
      el.addEventListener('pointerdown', onPointerDown as EventListener);
      el.addEventListener('pointermove', onPointerMove as EventListener);
      el.addEventListener('pointerup', onPointerUp as EventListener);
      el.addEventListener('pointercancel', cancel as EventListener);
    }
  }

  // Watch for target ref changes to (re)attach
  watch(target, (el) => attachTo(el), { immediate: true });

  onUnmounted(() => {
    attachTo(null); // Detach
    if (timer) clearTimeout(timer);
  });

  return { isPressed };
}
