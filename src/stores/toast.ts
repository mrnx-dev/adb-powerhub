import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface ToastItem {
  id: number;
  message: string;
  type: 'success' | 'error' | 'info' | 'progress';
  timestamp: number;
  progress?: number;
  action?: { label: string; onClick: () => void };
}

let nextId = 0;

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<ToastItem[]>([]);
  const MAX_TOASTS = 5;

  function show(message: string, type: ToastItem['type'] = 'info', duration = 3000) {
    const id = nextId++;
    const toast: ToastItem = { id, message, type, timestamp: Date.now() };
    toasts.value.push(toast);
    if (toasts.value.length > MAX_TOASTS) {
      toasts.value.shift();
    }
    if (duration > 0 && type !== 'progress') {
      setTimeout(() => remove(id), duration);
    }
    return id;
  }

  function updateProgress(id: number, percent: number) {
    const t = toasts.value.find((x) => x.id === id);
    if (t) t.progress = percent;
  }

  function remove(id: number) {
    const idx = toasts.value.findIndex((t) => t.id === id);
    if (idx !== -1) {
      toasts.value.splice(idx, 1);
    }
  }

  return { toasts, show, remove, updateProgress };
});
