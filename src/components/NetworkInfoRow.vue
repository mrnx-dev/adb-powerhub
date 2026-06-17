<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Wifi, WifiOff, Signal, ShieldCheck, Copy } from '@lucide/vue';
import { useToastStore } from '@/stores/toast';
import NetworkInfoTooltip from '@/components/NetworkInfoTooltip.vue';

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
  } | null;
  loading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
});

const toast = useToastStore();
const tooltipVisible = ref(false);
const hoverTimeout = ref<ReturnType<typeof setTimeout> | null>(null);
const rowRef = ref<HTMLElement | null>(null);
const tooltipStyle = ref<Record<string, string>>({});
const originX = ref('0');
const originY = ref('0');

const hasNetwork = computed(() => props.network !== null);
const isWifi = computed(() => props.network?.network_type === 'Wi-Fi');
const isMobile = computed(() => props.network?.network_type === 'Mobile');
const isVpn = computed(() => props.network?.network_type === 'VPN');
const hasAssociation = computed(() => {
  const n = props.network;
  if (!n) return false;
  const hasSsid =
    !!n.ssid && n.ssid.trim() !== '' && n.ssid.trim().toLowerCase() !== '<unknown ssid>';
  const hasBssid = !!n.bssid && n.bssid.trim() !== '' && n.bssid.trim() !== '00:00:00:00:00:00';
  return hasSsid || hasBssid;
});

// Only the connected Wi-Fi variant carries rich detail worth a tooltip + copy;
// all other variants are static status rows (non-interactive, tabindex -1).
const hasTooltip = computed(() => isWifi.value && hasAssociation.value);

interface SignalQuality {
  label: string;
  classes: string;
}

const signalQuality = computed<SignalQuality>(() => {
  const dbm = props.network?.signal_dbm;
  if (dbm === undefined || dbm === null || dbm >= 0) {
    return { label: '', classes: 'bg-theme-btn text-theme-muted' };
  }
  if (dbm >= -50)
    return { label: 'Excellent', classes: 'bg-color-success-container text-color-success' };
  if (dbm >= -60)
    return { label: 'Good', classes: 'bg-color-success-container/50 text-color-success' };
  if (dbm >= -70)
    return { label: 'Fair', classes: 'bg-color-warning-container text-color-warning' };
  return { label: 'Weak', classes: 'bg-color-error-container text-color-error' };
});

const primaryText = computed(() => {
  if (isWifi.value) return hasAssociation.value ? null : 'Wi-Fi on · not connected';
  if (isMobile.value || isVpn.value) return null;
  return 'No active network · —';
});

const displaySsid = computed(() => {
  const ssid = props.network?.ssid;
  if (!ssid || ssid.trim() === '' || ssid.trim().toLowerCase().startsWith('<unknown'))
    return 'Hidden Network';
  return ssid;
});

function isValidString(value: string | undefined): boolean {
  if (!value) return false;
  const v = value.trim();
  return v !== '' && v.toLowerCase() !== 'null' && v !== '0.0.0.0';
}

function formatValue(value: string | number | undefined): string {
  if (value === undefined || value === null || value === '') return '—';
  const s = String(value).trim();
  if (s === '' || s.toLowerCase() === 'null' || s === '0.0.0.0') return '—';
  return s;
}

function buildCopyText(): string {
  const n = props.network;
  if (!n) return '';
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
  const text = buildCopyText();
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    toast.show('Network info copied to clipboard', 'success');
  } catch {
    toast.show('Failed to copy network info', 'error');
  }
}

function updateTooltipPosition() {
  if (!rowRef.value) return;
  const rect = rowRef.value.getBoundingClientRect();
  const triggerCenterX = rect.left + rect.width / 2;
  const triggerBottomY = rect.bottom;
  tooltipStyle.value = {
    position: 'fixed',
    top: `${rect.bottom + 8}px`,
    left: `${rect.left}px`,
    minWidth: '16rem',
    maxWidth: '16rem',
  };
  originX.value = `${triggerCenterX}px`;
  originY.value = `${triggerBottomY}px`;
}

function showTooltip() {
  if (!hasTooltip.value) return;
  if (hoverTimeout.value) clearTimeout(hoverTimeout.value);
  hoverTimeout.value = setTimeout(() => {
    updateTooltipPosition();
    tooltipVisible.value = true;
  }, 150);
}

function hideTooltip() {
  if (hoverTimeout.value) clearTimeout(hoverTimeout.value);
  tooltipVisible.value = false;
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') hideTooltip();
}

function handleClickOutside(e: MouseEvent) {
  const target = e.target as Node;
  if (rowRef.value && !rowRef.value.contains(target)) {
    tooltipVisible.value = false;
  }
}

function handleScroll() {
  if (tooltipVisible.value) tooltipVisible.value = false;
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
  window.addEventListener('scroll', handleScroll, true);
  window.addEventListener('resize', handleScroll, true);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
  window.removeEventListener('scroll', handleScroll, true);
  window.removeEventListener('resize', handleScroll, true);
  if (hoverTimeout.value) clearTimeout(hoverTimeout.value);
});
</script>

<template>
  <div
    v-if="loading || hasNetwork"
    ref="rowRef"
    :tabindex="hasTooltip ? 0 : -1"
    :aria-describedby="hasTooltip ? 'network-info-tooltip' : undefined"
    class="relative flex items-center gap-2 mt-2 py-1.5 rounded-md focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-emerald/40"
    :class="[
      isWifi && hasAssociation
        ? 'text-theme-primary'
        : isMobile || isVpn
          ? 'text-theme-secondary'
          : 'text-theme-muted',
      'network-row-hover',
      hasTooltip ? 'cursor-help' : 'cursor-default',
    ]"
    @mouseenter="showTooltip"
    @mouseleave="hideTooltip"
    @focus="showTooltip"
    @blur="hideTooltip"
    @keydown="handleKeydown"
  >
    <!-- Loading skeleton -->
    <template v-if="loading">
      <div class="h-2 rounded-full network-row-skeleton w-full max-w-[180px]"></div>
    </template>

    <!-- Status messages: "Wi-Fi on · not connected" / "No active network · —" -->
    <template v-else-if="primaryText">
      <WifiOff :size="13" class="shrink-0" />
      <span class="text-[11px]">{{ primaryText }}</span>
    </template>

    <!-- Mobile data -->
    <template v-else-if="isMobile">
      <Signal :size="13" class="shrink-0 text-theme-secondary" />
      <span class="text-[11px] font-medium">Mobile data</span>
      <span
        v-if="network && isValidString(network.ip_address)"
        class="text-[10px] text-theme-secondary shrink-0"
        >{{ network.ip_address }}</span
      >
    </template>

    <!-- VPN -->
    <template v-else-if="isVpn">
      <ShieldCheck :size="13" class="shrink-0 text-theme-secondary" />
      <span class="text-[11px] font-medium">VPN</span>
      <span
        v-if="network && isValidString(network.ip_address)"
        class="text-[10px] text-theme-secondary shrink-0"
        >{{ network.ip_address }}</span
      >
    </template>

    <!-- Connected Wi-Fi info -->
    <template v-else>
      <button
        type="button"
        class="network-copy-btn btn-pressable shrink-0 flex items-center justify-center w-5 h-5 rounded-md text-theme-muted hover:text-theme-primary focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-emerald/40"
        aria-label="Copy network info to clipboard"
        @click="copyNetworkInfo"
      >
        <Copy :size="12" />
      </button>
      <Wifi :size="13" class="shrink-0 text-accent-emerald" />
      <span class="text-[11px] font-medium truncate max-w-[140px]">{{ displaySsid }}</span>

      <span
        v-if="signalQuality.label"
        class="text-[10px] px-1.5 py-0.5 rounded-full shrink-0"
        :class="signalQuality.classes"
      >
        {{ signalQuality.label }}
      </span>

      <span
        v-if="network?.link_speed_mbps"
        class="text-[10px] text-theme-secondary whitespace-nowrap shrink-0"
      >
        {{ network.link_speed_mbps }} Mbps
      </span>

      <span v-if="network?.network_type" class="text-[10px] text-theme-secondary shrink-0">{{
        network.network_type
      }}</span>

      <span
        v-if="network && isValidString(network.ip_address)"
        class="text-[10px] text-theme-secondary shrink-0"
        >{{ network.ip_address }}</span
      >
    </template>

    <Teleport v-if="network && hasTooltip" to="body">
      <NetworkInfoTooltip
        id="network-info-tooltip"
        :network="network"
        :visible="tooltipVisible"
        :origin-x="originX"
        :origin-y="originY"
        :position-style="tooltipStyle"
      />
    </Teleport>
  </div>
</template>

<style scoped>
.network-row-hover {
  transition:
    background-color 200ms var(--ease-out),
    transform 120ms var(--ease-out);
}

.network-copy-btn:hover {
  background-color: color-mix(in srgb, var(--theme-hover) 30%, transparent);
}

@media (hover: hover) and (pointer: fine) {
  .network-row-hover:hover {
    background-color: color-mix(in srgb, var(--theme-hover) 30%, transparent);
  }
}

@media (prefers-reduced-motion: reduce) {
  .network-row-hover {
    transition-duration: 0.01ms !important;
    transform: none !important;
  }
}
</style>
