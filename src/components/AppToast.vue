<script setup lang="ts">
import { ref } from 'vue';
import { useToastStore } from '../stores/toast';
import { Check, X, Info, Download } from '@lucide/vue';

const toastStore = useToastStore();
const dismissing = ref<number | null>(null);

const typeStyles: Record<string, string> = {
  success: 'border-l-accent-emerald bg-accent-light',
  error: 'border-l-color-error bg-color-error-container',
  info: 'border-l-color-info bg-color-info-container',
  progress: 'border-l-accent-emerald bg-theme-btn',
};

const typeIcons: Record<string, typeof Check> = {
  success: Check,
  error: X,
  info: Info,
  progress: Download,
};

const typeIconColors: Record<string, string> = {
  success: 'text-accent-emerald',
  error: 'text-color-error',
  info: 'text-color-info',
  progress: 'text-accent-emerald',
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
  <div
    class="fixed bottom-4 right-4 z-40 flex flex-col gap-2 pointer-events-none"
    style="max-width: 360px"
  >
    <TransitionGroup name="toast">
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        class="pointer-events-auto flex flex-col gap-2 px-4 py-3 rounded-xl border-l-4 backdrop-blur-md shadow-theme-card text-sm transition-[opacity,transform] duration-150"
        :class="[typeStyles[toast.type], { 'scale-95 opacity-0': dismissing === toast.id }]"
        @click="onDismiss(toast.id)"
      >
        <div class="flex items-start gap-2.5">
          <component
            :is="typeIcons[toast.type]"
            :size="16"
            class="shrink-0 mt-0.5"
            :class="typeIconColors[toast.type]"
          />
          <span class="text-theme-primary leading-snug">{{ toast.message }}</span>
          <button
            v-if="toast.action"
            class="ml-2 px-2 py-0.5 rounded-full text-xs font-medium bg-accent-emerald/20 text-accent-emerald hover:bg-accent-emerald/30 transition-colors whitespace-nowrap self-center"
            :aria-label="toast.action.label"
            @click.stop="
              toast.action!.onClick();
              toastStore.remove(toast.id);
            "
          >
            {{ toast.action.label }}
          </button>
        </div>
        <!-- Progress bar (only for progress toasts) -->
        <div v-if="toast.type === 'progress' && toast.progress !== undefined" class="w-full">
          <div class="flex items-center justify-between mb-1">
            <span class="text-[10px] text-theme-secondary">{{ toast.progress }}%</span>
          </div>
          <div class="progress-bar-track">
            <div class="progress-bar-fill" :style="{ width: toast.progress + '%' }"></div>
          </div>
        </div>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-enter-active {
  transition:
    opacity var(--duration-standard) var(--ease-out),
    transform var(--duration-standard) var(--ease-out);
}
.toast-leave-active {
  transition:
    opacity 100ms var(--ease-accelerate),
    transform 100ms var(--ease-accelerate);
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
