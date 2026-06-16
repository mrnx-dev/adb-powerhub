<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Wifi, WifiOff } from '@lucide/vue';
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

const tooltipVisible = ref(false);
const hoverTimeout = ref<ReturnType<typeof setTimeout> | null>(null);
const rowRef = ref<HTMLElement | null>(null);
const isPointerDown = ref(false);

const hasNetwork = computed(() => props.network !== null);
const isWifi = computed(() => props.network?.network_type === 'Wi-Fi');
const hasAssociation = computed(() => {
  const n = props.network;
  if (!n) return false;
  const hasSsid =
    !!n.ssid && n.ssid.trim() !== '' && n.ssid.trim().toLowerCase() !== '<unknown ssid>';
  const hasBssid = !!n.bssid && n.bssid.trim() !== '' && n.bssid.trim() !== '00:00:00:00:00:00';
  return hasSsid || hasBssid;
});

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
  if (!isWifi.value) return 'Wi-Fi disabled · —';
  if (!hasAssociation.value) return 'Wi-Fi on · not connected';
  return null;
});

const displaySsid = computed(() => {
  const ssid = props.network?.ssid;
  if (!ssid || ssid.trim() === '' || ssid.trim().toLowerCase() === '<unknown ssid>')
    return 'Hidden Network';
  return ssid;
});

function showTooltip() {
  // Ignore focus events caused by a pointer click; the click handler will toggle.
  if (isPointerDown.value) {
    isPointerDown.value = false;
    return;
  }
  if (hoverTimeout.value) clearTimeout(hoverTimeout.value);
  hoverTimeout.value = setTimeout(() => {
    tooltipVisible.value = true;
  }, 150);
}

function hideTooltip() {
  if (hoverTimeout.value) clearTimeout(hoverTimeout.value);
  tooltipVisible.value = false;
  isPointerDown.value = false;
}

function toggleTooltip() {
  if (hoverTimeout.value) clearTimeout(hoverTimeout.value);
  tooltipVisible.value = !tooltipVisible.value;
  isPointerDown.value = false;
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    hideTooltip();
  } else if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault();
    toggleTooltip();
  }
}

function handleClickOutside(e: MouseEvent) {
  if (rowRef.value && !rowRef.value.contains(e.target as Node)) {
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
    tabindex="0"
    role="button"
    aria-describedby="network-info-tooltip"
    class="relative flex items-center gap-2 mt-2 px-2 py-1.5 rounded-md transition-colors duration-200 cursor-help focus:outline-none focus:ring-2 focus:ring-accent-emerald/40"
    :class="[
      isWifi && hasAssociation ? 'text-theme-primary' : 'text-theme-muted',
      'hover:bg-theme-hover/30',
    ]"
    @mouseenter="showTooltip"
    @mouseleave="hideTooltip"
    @focus="showTooltip"
    @blur="hideTooltip"
    @pointerdown="isPointerDown = true"
    @click="toggleTooltip"
    @keydown="handleKeydown"
  >
    <!-- Loading skeleton -->
    <template v-if="loading">
      <div class="h-2 rounded-full network-row-skeleton w-full max-w-[180px]"></div>
    </template>

    <!-- Disconnected / Wi-Fi disabled -->
    <template v-else-if="primaryText">
      <WifiOff :size="13" class="shrink-0" />
      <span class="text-[11px]">{{ primaryText }}</span>
    </template>

    <!-- Connected Wi-Fi info -->
    <template v-else>
      <Wifi :size="13" class="shrink-0 text-accent-emerald" />
      <span class="text-[11px] font-medium truncate max-w-[140px]">{{ displaySsid }}</span>

      <span
        v-if="signalQuality.label"
        class="text-[10px] px-1.5 py-0.5 rounded-full shrink-0 transition-colors duration-150"
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

      <span v-if="network?.ip_address" class="text-[10px] text-theme-secondary shrink-0">{{
        network.ip_address
      }}</span>
    </template>

    <NetworkInfoTooltip
      v-if="network"
      id="network-info-tooltip"
      :network="network"
      :visible="tooltipVisible"
    />
  </div>
</template>

<style scoped>
@media (hover: hover) and (pointer: fine) {
  .hover\:bg-theme-hover\/30:hover {
    background-color: color-mix(in srgb, var(--theme-hover) 30%, transparent);
  }
}

@media (prefers-reduced-motion: reduce) {
  .transition-colors,
  .duration-200,
  .duration-150 {
    transition-duration: 0.01ms !important;
  }
}
</style>
