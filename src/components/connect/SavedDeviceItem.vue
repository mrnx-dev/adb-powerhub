<script setup lang="ts">
import { Smartphone, Trash2 } from '@lucide/vue';

defineProps<{
  device: {
    id: string;
    ip: string;
    port: number;
    label: string;
    method: 'wifi' | 'pairing';
    lastConnected: string;
  };
  isCurrent: boolean;
  index: number;
}>();

const emit = defineEmits<{
  connect: [];
  forget: [];
}>();
</script>

<template>
  <div
    class="flex items-start gap-3 py-3 px-3 rounded-lg border-l-4 border-accent-emerald bg-theme-hover/20 hover:bg-theme-hover/50 transition-colors"
  >
    <Smartphone :size="16" class="text-accent-emerald shrink-0 mt-0.5" />
    <div class="flex-1 min-w-0">
      <div class="text-xs font-semibold text-theme-primary truncate">
        {{ device.label }}
      </div>
      <div class="text-[10px] text-theme-muted mt-0.5">
        {{ device.ip }}:{{ device.port }}
        <span class="mx-1">&middot;</span>
        {{ device.method }}
      </div>
      <div class="text-[10px] text-theme-muted">
        Last: {{ new Date(device.lastConnected).toLocaleDateString() }}
      </div>
    </div>
    <div class="flex items-center gap-1.5 shrink-0">
      <button
        class="btn-primary text-[10px] py-1 px-2.5 rounded-md font-semibold disabled:opacity-50"
        :disabled="isCurrent"
        :aria-label="`Connect to ${device.label}`"
        @click="emit('connect')"
      >
        Connect
      </button>
      <button
        class="p-1.5 rounded-md text-theme-muted hover:text-color-error hover:bg-color-error-container transition-colors"
        :aria-label="`Forget ${device.label}`"
        title="Forget device"
        @click="emit('forget')"
      >
        <Trash2 :size="13" />
      </button>
    </div>
  </div>
</template>
