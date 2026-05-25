import { defineStore } from "pinia";
import { ref } from "vue";

export interface ToastItem {
  id: number;
  message: string;
  type: "success" | "error" | "info";
  timestamp: number;
}

let nextId = 0;

export const useToastStore = defineStore("toast", () => {
  const toasts = ref<ToastItem[]>([]);
  const MAX_TOASTS = 5;

  function show(message: string, type: "success" | "error" | "info" = "info", duration = 3000) {
    const id = nextId++;
    const toast: ToastItem = { id, message, type, timestamp: Date.now() };
    toasts.value.push(toast);
    if (toasts.value.length > MAX_TOASTS) {
      toasts.value.shift();
    }
    if (duration > 0) {
      setTimeout(() => remove(id), duration);
    }
  }

  function remove(id: number) {
    const idx = toasts.value.findIndex((t) => t.id === id);
    if (idx !== -1) {
      toasts.value.splice(idx, 1);
    }
  }

  return { toasts, show, remove };
});