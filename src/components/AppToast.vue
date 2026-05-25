<script setup lang="ts">
import { useToastStore } from "../stores/toast";
import { Check, X, Info } from "lucide-vue-next";

const toastStore = useToastStore();

const typeStyles: Record<string, string> = {
  success: "border-l-emerald-500 bg-emerald-500/10",
  error: "border-l-red-500 bg-red-500/10",
  info: "border-l-gray-400 bg-white/5",
};

const typeIcons: Record<string, typeof Check> = {
  success: Check,
  error: X,
  info: Info,
};

const typeIconColors: Record<string, string> = {
  success: "text-emerald-400",
  error: "text-red-400",
  info: "text-gray-400",
};
</script>

<template>
  <div class="fixed bottom-4 right-4 z-40 flex flex-col gap-2 pointer-events-none" style="max-width: 360px;">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        class="pointer-events-auto flex items-start gap-2.5 px-4 py-3 rounded-xl border-l-4 backdrop-blur-md shadow-lg text-sm"
        :class="typeStyles[toast.type]"
        @click="toastStore.remove(toast.id)">
        <component
          :is="typeIcons[toast.type]"
          :size="16"
          class="shrink-0 mt-0.5"
          :class="typeIconColors[toast.type]" />
        <span class="text-gray-200 leading-snug">{{ toast.message }}</span>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-enter-active {
  transition: all 0.25s ease-out;
}
.toast-leave-active {
  transition: all 0.2s ease-in;
}
.toast-enter-from {
  opacity: 0;
  transform: translateX(1rem);
}
.toast-leave-to {
  opacity: 0;
  transform: translateX(1rem);
}
.toast-move {
  transition: transform 0.25s ease;
}
</style>