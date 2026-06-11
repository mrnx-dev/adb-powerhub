<script setup lang="ts">
import { XCircle } from '@lucide/vue';

defineProps<{
  message: string;
}>();
</script>

<template>
  <div
    role="alert"
    aria-live="assertive"
    class="flex items-center gap-2 py-2.5 px-3 rounded-lg bg-color-error-container border border-color-error"
  >
    <XCircle :size="14" class="text-color-error shrink-0" />
    <span class="text-[11px] text-color-error">{{ message }}</span>
  </div>
</template>
