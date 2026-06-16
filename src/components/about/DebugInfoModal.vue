<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';
import { useAboutStore } from '../../stores/about';

const open = defineModel<boolean>('open', { default: false });

const aboutStore = useAboutStore();
const modalRef = ref<HTMLElement | null>(null);
const closeButtonRef = ref<HTMLButtonElement | null>(null);
const copyHintVisible = ref(false);
const copyError = ref('');
const loading = ref(false);

async function loadDebugInfo() {
  if (!open.value) return;
  loading.value = true;
  copyHintVisible.value = false;
  copyError.value = '';
  await aboutStore.getDebugInfo();
  loading.value = false;
}

watch(open, (isOpen) => {
  if (isOpen) {
    loadDebugInfo();
    nextTick(() => {
      closeButtonRef.value?.focus();
    });
    document.addEventListener('keydown', onKeyDown);
  } else {
    document.removeEventListener('keydown', onKeyDown);
  }
});

function closeModal() {
  open.value = false;
}

function onBackdropClick(event: MouseEvent) {
  if (event.target === modalRef.value) {
    closeModal();
  }
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    closeModal();
    return;
  }
  if (event.key === 'Tab' && modalRef.value) {
    const focusable = modalRef.value.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (focusable.length === 0) return;
    const first = focusable[0] as HTMLElement;
    const last = focusable[focusable.length - 1] as HTMLElement;
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
}

async function copyToClipboard() {
  const text = formattedDebugInfo.value;
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    copyHintVisible.value = true;
    copyError.value = '';
    setTimeout(() => {
      copyHintVisible.value = false;
    }, 2000);
  } catch {
    copyError.value = 'Failed to copy to clipboard';
    copyHintVisible.value = false;
  }
}

const formattedDebugInfo = ref('');
watch(
  () => aboutStore.debugInfo,
  (info) => {
    if (info) {
      formattedDebugInfo.value = JSON.stringify(info, null, 2);
    } else {
      formattedDebugInfo.value = '';
    }
  },
  { immediate: true }
);
</script>

<template>
  <Transition name="modal-fade">
    <div
      v-if="open"
      ref="modalRef"
      class="modal-backdrop fixed inset-0 z-[100] flex items-center justify-center p-4"
      style="background: rgba(0, 0, 0, 0.55); backdrop-filter: blur(6px)"
      @click="onBackdropClick"
    >
      <div
        class="modal w-full max-w-[520px] max-h-[80vh] bg-theme-sidebar border-2 border-theme-primary rounded-lg shadow-theme-modal flex flex-col"
        role="dialog"
        aria-modal="true"
        aria-labelledby="modal-title"
        @click.stop
      >
        <div
          class="modal-header flex items-center justify-between px-5 py-4 border-b border-theme-tertiary"
        >
          <h3 id="modal-title" class="text-[15px] text-theme-heading m-0">Debug Info</h3>
          <button
            ref="closeButtonRef"
            type="button"
            class="modal-close w-7 h-7 rounded-md border-0 bg-transparent text-theme-muted flex items-center justify-center btn-pressable"
            aria-label="Close dialog"
            @click="closeModal"
          >
            ✕
          </button>
        </div>

        <div class="modal-body px-5 py-4 overflow-y-auto">
          <div v-if="loading" class="shimmer h-20 rounded-md"></div>
          <pre
            v-else
            class="debug-code bg-theme-card border border-theme-tertiary rounded-md p-3 font-mono text-[11px] leading-relaxed text-theme-secondary whitespace-pre-wrap break-words max-h-[320px] overflow-y-auto"
            >{{ formattedDebugInfo }}</pre
          >

          <div class="flex gap-3 mt-3 justify-end">
            <button type="button" class="btn btn-pressable" @click="closeModal">Close</button>
            <button
              type="button"
              class="btn btn-primary btn-pressable"
              :disabled="!formattedDebugInfo || loading"
              @click="copyToClipboard"
            >
              Copy to Clipboard
            </button>
          </div>

          <div
            :class="[
              'copy-hint text-xs text-color-success mt-2 transition-opacity duration-200',
              copyHintVisible ? 'opacity-100' : 'opacity-0',
            ]"
          >
            Copied to clipboard
          </div>
          <div v-if="copyError" class="text-xs text-color-error mt-2">{{ copyError }}</div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity var(--duration-standard) var(--ease-out);
}
.modal-fade-enter-active .modal,
.modal-fade-leave-active .modal {
  transition: transform var(--duration-standard) var(--ease-emphasized);
}
.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}
.modal-fade-enter-from .modal,
.modal-fade-leave-to .modal {
  transform: translateY(16px) scale(0.97);
}

.shimmer {
  background: linear-gradient(
    90deg,
    var(--bg-card) 25%,
    rgba(255, 255, 255, 0.05) 50%,
    var(--bg-card) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s ease-in-out infinite;
}

@media (prefers-reduced-motion: reduce) {
  .shimmer {
    animation: none;
    background: var(--bg-card);
  }
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  .modal-fade-enter-active,
  .modal-fade-leave-active,
  .modal-fade-enter-active .modal,
  .modal-fade-leave-active .modal {
    transition: none;
  }
  .modal-fade-enter-from .modal,
  .modal-fade-leave-to .modal {
    transform: none;
  }
}
</style>
