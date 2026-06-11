<script setup lang="ts">
import { computed } from 'vue';
import { Loader2 } from '@lucide/vue';

const props = defineProps<{
  status:
    | 'idle'
    | 'detecting_usb'
    | 'enabling_tcp'
    | 'detecting_ip'
    | 'connecting_tcp'
    | 'connected'
    | 'error';
}>();

const label = computed(() => {
  switch (props.status) {
    case 'detecting_usb':
      return 'Looking for device\u2026';
    case 'enabling_tcp':
      return 'Enabling wireless mode\u2026';
    case 'detecting_ip':
      return 'Detecting IP address\u2026';
    case 'connecting_tcp':
      return 'Connecting via Wi-Fi\u2026';
    default:
      return 'Connecting\u2026'; // defensive fallback (FR-2 edge)
  }
});
</script>

<template>
  <div
    role="status"
    aria-live="polite"
    aria-atomic="true"
    class="flex items-center gap-2 py-2 px-3 rounded-md bg-accent-light text-[11px] text-accent-emerald"
  >
    <Loader2 :size="14" class="animate-spin shrink-0" />
    <span>{{ label }}</span>
  </div>
</template>
