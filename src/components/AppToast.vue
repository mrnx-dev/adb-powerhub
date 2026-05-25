<script setup lang="ts">
import { ref } from "vue";
import { useToastStore } from "../stores/toast";
import { Check, X, Info, Download } from "lucide-vue-next";

const toastStore = useToastStore();
const dismissing = ref<number | null>(null);

const typeStyles: Record<string, string> = {
  success: "border-l-emerald-500 bg-emerald-500/10",
  error: "border-l-red-500 bg-red-500/10",
  info: "border-l-gray-400 bg-white/5",
  progress: "border-l-emerald-500 bg-white/5",
};

const typeIcons: Record<string, typeof Check> = {
  success: Check,
  error: X,
  info: Info,
  progress: Download,
};

const typeIconColors: Record<string, string> = {
  success: "text-emerald-400",
  error: "text-red-400",
  info: "text-gray-400",
  progress: "text-accent-emerald",
};

function onDismiss(id: number) {
  dismissing.value = id;
  setTimeout(() => {
    toastStore.remove(id);
    dismissing.value = null;
  }, 150);
}
</script>

<template>
  <div class="fixed bottom-4 right-4 z-40 flex flex-col gap-2 pointer-events-none" style="max-width: 360px;">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        class="pointer-events-auto flex flex-col gap-2 px-4 py-3 rounded-xl border-l-4 backdrop-blur-md shadow-lg text-sm transition-all duration-150"
        :class="[
          typeStyles[toast.type],
          { 'scale-95 opacity-0': dismissing === toast.id }
        ]"
        @click="onDismiss(toast.id)">
        <div class="flex items-start gap-2.5">
          <component
            :is="typeIcons[toast.type]"
            :size="16"
            class="shrink-0 mt-0.5"
            :class="typeIconColors[toast.type]" />
          <span class="text-gray-200 leading-snug">{{ toast.message }}</span>
        </div>
        <!-- Progress bar (only for progress toasts) -->
        <div v-if="toast.type === 'progress' && toast.progress !== undefined" class="w-full">
          <div class="flex items-center justify-between mb-1">
            <span class="text-[10px] text-gray-400">{{ toast.progress }}%</span>
          </div>
          <div class="w-full bg-gray-700 rounded-full h-1.5">
            <div class="bg-accent-emerald h-1.5 rounded-full transition-all duration-300"
              :style="{ width: toast.progress + '%' }"></div>
          </div>
        </div>
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