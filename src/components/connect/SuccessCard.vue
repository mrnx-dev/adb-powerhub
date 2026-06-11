<script setup lang="ts">
import { Check } from '@lucide/vue';

defineProps<{
  deviceModel: string;
  deviceId: string;
  transport: 'usb' | 'wifi';
}>();
</script>

<template>
  <div role="status" aria-live="polite" class="space-y-3">
    <div
      class="flex items-center gap-2 py-2.5 px-3 rounded-lg bg-accent-light border border-accent-default"
    >
      <Check :size="14" class="text-accent-emerald shrink-0" />
      <div>
        <div class="text-xs font-semibold text-accent-emerald">
          {{ deviceModel || deviceId }}
        </div>
        <div class="text-[10px] text-theme-muted mt-0.5">
          {{ transport === 'wifi' ? 'Wi-Fi' : 'USB' }}
          <span class="mx-0.5">&middot;</span>
          {{ deviceId }}
        </div>
      </div>
    </div>
    <div v-if="transport === 'wifi'" class="text-[10px] text-theme-muted px-1">
      You may unplug USB safely
    </div>
    <div v-else class="text-[10px] text-theme-muted px-1">
      Unplugging USB will disconnect the device
    </div>
  </div>
</template>
