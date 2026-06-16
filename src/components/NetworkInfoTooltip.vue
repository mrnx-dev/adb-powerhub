<script setup lang="ts">
import { Wifi, Signal, Router, Cable, Globe, Shield, Smartphone, Activity } from '@lucide/vue';

interface Props {
  network: {
    ssid?: string;
    bssid?: string;
    signal_dbm?: number;
    link_speed_mbps?: number;
    frequency_mhz?: number;
    device_mac?: string;
    http_proxy?: string;
    network_type?: string;
    ip_address?: string;
  };
  visible: boolean;
  id?: string;
  originX?: string;
  originY?: string;
  positionStyle?: Record<string, string>;
}

withDefaults(defineProps<Props>(), {
  originX: '0',
  originY: '0',
  id: undefined,
  positionStyle: undefined,
});

function formatValue(value: string | number | undefined): string {
  if (value === undefined || value === null || value === '') return '—';
  return String(value);
}
</script>

<template>
  <Transition
    enter-active-class="transition duration-200 ease-emphasized"
    enter-from-class="opacity-0 scale-[0.97]"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition duration-150 ease-emphasized"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-[0.97]"
  >
    <div
      v-if="visible"
      :id="id"
      role="tooltip"
      class="w-64 rounded-lg border border-theme-secondary bg-theme-sidebar shadow-theme-modal p-3 text-[11px]"
      :class="[positionStyle ? 'fixed z-[100]' : 'absolute z-50 top-full left-0 mt-2']"
      :style="{
        ...(positionStyle ?? {}),
        transformOrigin: `${originX} ${originY}`,
      }"
    >
      <div class="flex items-center gap-2 mb-2 pb-2 border-b border-theme-tertiary">
        <Wifi :size="13" class="text-accent-emerald" />
        <span class="font-semibold text-theme-primary">Network Details</span>
      </div>

      <div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5">
        <span class="text-theme-secondary flex items-center gap-1.5">
          <Smartphone :size="11" /> SSID
        </span>
        <span class="text-theme-primary text-right truncate" :title="formatValue(network.ssid)">{{
          formatValue(network.ssid)
        }}</span>

        <span class="text-theme-secondary flex items-center gap-1.5">
          <Router :size="11" /> BSSID
        </span>
        <span class="text-theme-primary text-right truncate" :title="formatValue(network.bssid)">{{
          formatValue(network.bssid)
        }}</span>

        <span class="text-theme-secondary flex items-center gap-1.5">
          <Signal :size="11" /> Signal
        </span>
        <span class="text-theme-primary text-right">{{
          network.signal_dbm !== undefined ? `${network.signal_dbm} dBm` : '—'
        }}</span>

        <span class="text-theme-secondary flex items-center gap-1.5">
          <Activity :size="11" /> Link Speed
        </span>
        <span class="text-theme-primary text-right">{{
          network.link_speed_mbps !== undefined ? `${network.link_speed_mbps} Mbps` : '—'
        }}</span>

        <span class="text-theme-secondary flex items-center gap-1.5">
          <Globe :size="11" /> Frequency
        </span>
        <span class="text-theme-primary text-right">{{
          network.frequency_mhz !== undefined ? `${network.frequency_mhz} MHz` : '—'
        }}</span>

        <span class="text-theme-secondary flex items-center gap-1.5">
          <Cable :size="11" /> Device MAC
        </span>
        <span
          class="text-theme-primary text-right truncate"
          :title="formatValue(network.device_mac)"
          >{{ formatValue(network.device_mac) }}</span
        >

        <span class="text-theme-secondary flex items-center gap-1.5">
          <Shield :size="11" /> HTTP Proxy
        </span>
        <span
          class="text-theme-primary text-right truncate"
          :title="formatValue(network.http_proxy)"
          >{{ formatValue(network.http_proxy) }}</span
        >

        <span class="text-theme-secondary flex items-center gap-1.5">
          <Globe :size="11" /> Type
        </span>
        <span class="text-theme-primary text-right">{{ formatValue(network.network_type) }}</span>

        <span class="text-theme-secondary flex items-center gap-1.5">
          <Wifi :size="11" /> IP Address
        </span>
        <span
          class="text-theme-primary text-right truncate"
          :title="formatValue(network.ip_address)"
          >{{ formatValue(network.ip_address) }}</span
        >
      </div>
    </div>
  </Transition>
</template>

<style scoped>
@media (prefers-reduced-motion: reduce) {
  .ease-emphasized {
    transition-duration: 0.01ms !important;
  }
}
</style>
