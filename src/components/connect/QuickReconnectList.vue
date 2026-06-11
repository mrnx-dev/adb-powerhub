<script setup lang="ts">
import { Bookmark } from '@lucide/vue';
import SavedDeviceItem from './SavedDeviceItem.vue';

defineProps<{
  devices: {
    id: string;
    ip: string;
    port: number;
    label: string;
    method: 'wifi' | 'pairing';
    lastConnected: string;
  }[];
  currentDeviceId: string;
  connecting: boolean;
}>();

function deviceId(ip: string, port: number) {
  return `${ip}:${port}`;
}

const emit = defineEmits<{
  connect: [
    device: {
      ip: string;
      port: number;
      method: 'wifi' | 'pairing';
    },
  ];
  forget: [id: string];
}>();
</script>

<template>
  <div class="space-y-2.5">
    <!-- Section header -->
    <div class="flex items-center gap-2 pb-1">
      <Bookmark :size="14" class="text-theme-muted" />
      <span class="text-xs font-semibold text-theme-primary">Quick Reconnect</span>
    </div>

    <!-- Empty state (FR-3 AC3) -->
    <div v-if="devices.length === 0" class="text-center py-10">
      <Bookmark :size="28" class="mx-auto mb-2 opacity-20 text-theme-muted" />
      <p class="text-xs text-theme-muted">No saved devices yet</p>
      <p class="text-[10px] text-theme-muted mt-1">Devices will appear here after connecting</p>
    </div>

    <!-- Device list -->
    <TransitionGroup v-else name="list-stagger" tag="div" class="space-y-2">
      <SavedDeviceItem
        v-for="(device, index) in devices"
        :key="device.id"
        :device="device"
        :is-current="currentDeviceId === deviceId(device.ip, device.port)"
        :index="index"
        @connect="emit('connect', device)"
        @forget="emit('forget', device.id)"
      />
    </TransitionGroup>
  </div>
</template>
