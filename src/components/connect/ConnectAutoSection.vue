<script setup lang="ts">
import { Zap } from '@lucide/vue';
import StatusRow from './StatusRow.vue';
import SuccessCard from './SuccessCard.vue';
import ErrorBanner from './ErrorBanner.vue';

defineProps<{
  connecting: boolean;
  connected: boolean;
  status:
    | 'idle'
    | 'detecting_usb'
    | 'enabling_tcp'
    | 'detecting_ip'
    | 'connecting_tcp'
    | 'connected'
    | 'error';
  deviceModel: string;
  deviceId: string;
  transport: 'usb' | 'wifi';
  error: boolean;
  errorMessage: string;
}>();

const emit = defineEmits<{
  'auto-connect': [];
  disconnect: [];
}>();
</script>

<template>
  <div class="space-y-4 pt-2">
    <!-- CTA Area -->
    <div class="text-center py-6">
      <div
        class="w-12 h-12 rounded-2xl bg-accent-light border border-accent-default flex items-center justify-center mx-auto mb-3"
      >
        <Zap :size="24" class="text-accent-emerald" />
      </div>
      <p class="text-xs text-theme-secondary mb-4">Plug in USB or connect to an existing device</p>

      <!-- Auto Connect button (idle / connecting) -->
      <button
        v-if="!connected"
        :disabled="connecting"
        class="w-full btn-primary py-3 rounded-lg text-sm font-semibold disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        :class="{ 'animate-shimmer': connecting }"
        aria-label="Auto connect to device"
        :aria-disabled="connecting"
        @click="emit('auto-connect')"
      >
        <Zap :size="16" />
        <span>{{ connecting ? 'Connecting\u2026' : 'Auto Connect' }}</span>
      </button>

      <!-- Disconnect button (connected state, de-emphasized per FR-7 AC2) -->
      <button
        v-else
        class="w-full py-2.5 rounded-lg text-sm font-semibold border border-theme-tertiary text-theme-secondary hover:text-theme-primary hover:bg-theme-hover transition-colors flex items-center justify-center gap-2"
        @click="emit('disconnect')"
      >
        Disconnect
      </button>
    </div>

    <!-- Status Row (appears during connecting phase, FR-2) -->
    <Transition name="status-fade">
      <StatusRow v-if="connecting && !error && !connected" :status="status" />
    </Transition>

    <!-- Success Card (FR-5 AC5.1) -->
    <Transition name="card-fade">
      <SuccessCard
        v-if="connected"
        :device-model="deviceModel"
        :device-id="deviceId"
        :transport="transport"
      />
    </Transition>

    <!-- Error Banner (FR-5 AC5.2) -->
    <Transition name="card-fade">
      <ErrorBanner v-if="error" :message="errorMessage" />
    </Transition>
  </div>
</template>
