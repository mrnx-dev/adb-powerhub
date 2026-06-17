<script setup lang="ts">
import {
  Wifi,
  Signal,
  Router,
  Cable,
  Globe,
  Shield,
  Smartphone,
  Activity,
  Copy,
} from '@lucide/vue';
import { useToastStore } from '@/stores/toast';

const emit = defineEmits<{ (e: 'request-close'): void }>();
const toast = useToastStore();

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

const props = withDefaults(defineProps<Props>(), {
  originX: '0',
  originY: '0',
  id: undefined,
  positionStyle: undefined,
});

function formatValue(value: string | number | undefined): string {
  if (value === undefined || value === null || value === '') return '—';
  const s = String(value).trim();
  if (s === '' || s.toLowerCase() === 'null' || s === '0.0.0.0') return '—';
  return s;
}

function buildCopyText(): string {
  const n = props.network;
  const signal = n.signal_dbm !== undefined && n.signal_dbm !== null ? `${n.signal_dbm} dBm` : '—';
  const link =
    n.link_speed_mbps !== undefined && n.link_speed_mbps !== null
      ? `${n.link_speed_mbps} Mbps`
      : '—';
  const freq =
    n.frequency_mhz !== undefined && n.frequency_mhz !== null ? `${n.frequency_mhz} MHz` : '—';
  return [
    `SSID: ${formatValue(n.ssid)}`,
    `BSSID: ${formatValue(n.bssid)}`,
    `Signal: ${signal}`,
    `Link speed: ${link}`,
    `Frequency: ${freq}`,
    `Device MAC: ${formatValue(n.device_mac)}`,
    `HTTP proxy: ${formatValue(n.http_proxy)}`,
    `Network type: ${formatValue(n.network_type)}`,
    `IP address: ${formatValue(n.ip_address)}`,
  ].join('\n');
}

async function copyNetworkInfo() {
  try {
    await navigator.clipboard.writeText(buildCopyText());
    toast.show('Network info copied to clipboard', 'success');
  } catch {
    toast.show('Failed to copy network info', 'error');
  }
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
      @keydown.escape="emit('request-close')"
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

      <div class="mt-2 pt-2 border-t border-theme-tertiary">
        <button
          type="button"
          class="network-copy-btn btn-pressable w-full flex items-center justify-center gap-1.5 px-2 py-1.5 rounded-md text-[11px] font-medium border border-theme-tertiary bg-theme-btn text-theme-secondary focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-emerald/40"
          aria-label="Copy network info to clipboard"
          @click="copyNetworkInfo"
        >
          <Copy :size="11" />
          <span>Copy</span>
        </button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.network-copy-btn:hover {
  background-color: color-mix(in srgb, var(--theme-hover) 30%, transparent);
}

@media (prefers-reduced-motion: reduce) {
  .ease-emphasized {
    transition-duration: 0.01ms !important;
  }
}
</style>
