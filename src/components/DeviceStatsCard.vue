<script setup lang="ts">
import { useDeviceStore } from '../stores/device';
import { useNavigationStore } from '../stores/navigation';
import { useDropdownRegistry } from '../composables/useDropdownRegistry';
import NetworkInfoRow from '@/components/NetworkInfoRow.vue';
import {
  Smartphone,
  Battery,
  Cpu,
  Zap,
  ChevronDown,
  RefreshCw,
  Unplug,
  HardDrive,
  MemoryStick,
  Thermometer,
} from '@lucide/vue';
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';

const store = useDeviceStore();
const nav = useNavigationStore();
const dropdown = useDropdownRegistry('device-stats-badge');
const dropdownOpen = ref(false);
const dropdownRef = ref<HTMLElement | null>(null);
const dropdownStyle = ref<Record<string, string>>({});
const badgeRef = ref<HTMLElement | null>(null);

function toggleDropdown() {
  dropdownOpen.value = !dropdownOpen.value;
  if (dropdownOpen.value) {
    dropdown.open();
    nextTick(updateDropdownPosition);
  } else {
    dropdown.close();
  }
}

/* FR-8: origin-aware dropdown. Teleport moves DOM to <body>, so
   transform-origin can't reference the trigger via CSS alone.
   Compute trigger position and set inline transform-origin. */
function updateDropdownPosition() {
  if (badgeRef.value) {
    const rect = badgeRef.value.getBoundingClientRect();
    const triggerCenterX = rect.left + rect.width / 2;
    const triggerBottomY = rect.bottom;
    dropdownStyle.value = {
      position: 'fixed',
      top: `${rect.bottom + 4}px`,
      left: `${rect.left}px`,
      minWidth: '160px',
      transformOrigin: `${triggerCenterX}px ${triggerBottomY}px`,
    };
  }
}

function handleClickOutside(e: MouseEvent) {
  const target = e.target as Node;
  if (
    dropdownRef.value &&
    !dropdownRef.value.contains(target) &&
    badgeRef.value &&
    !badgeRef.value.contains(target)
  ) {
    dropdownOpen.value = false;
  }
}

function handleScroll() {
  if (dropdownOpen.value) {
    dropdownOpen.value = false;
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
  window.addEventListener('scroll', handleScroll, true);
  // R5 mitigation: re-compute position on window resize to prevent drift
  window.addEventListener('resize', updateDropdownPosition);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
  window.removeEventListener('scroll', handleScroll, true);
  window.removeEventListener('resize', updateDropdownPosition);
  // P25/M25: ensure registry state is clean if component unmounts mid-open
  dropdown.close();
});

watch(dropdownOpen, (val) => {
  if (val) {
    dropdown.open();
    nextTick(updateDropdownPosition);
  } else {
    dropdown.close();
  }
});

function handleReconnect() {
  dropdownOpen.value = false;
  if (store.connected) {
    store.disconnect().then(() => {
      store.autoConnect();
    });
  } else {
    store.autoConnect();
  }
}

function handleDisconnect() {
  dropdownOpen.value = false;
  store.disconnect();
}
</script>

<template>
  <section class="card-glass p-4">
    <!-- Skeleton loading state -->
    <template v-if="store.connected && store.isLoadingStats">
      <div class="flex items-center justify-between">
        <div class="space-y-3 flex-1">
          <div class="h-2.5 bg-theme-btn rounded-full animate-pulse w-full max-w-[200px]"></div>
          <div class="h-2.5 bg-theme-btn rounded-full animate-pulse w-full max-w-[320px]"></div>
        </div>
      </div>
    </template>

    <!-- Connected state -->
    <template v-else-if="store.connected">
      <div class="flex items-start justify-between gap-4">
        <div class="flex-1 min-w-0">
          <!-- Line 1: connection meta -->
          <div class="flex items-center gap-2 text-[10px] text-theme-muted mb-1">
            <span class="capitalize">{{
              store.connectMethod === 'pairing' ? 'Android 11+ Pairing' : store.connectMethod
            }}</span>
            <span>&middot;</span>
            <span>{{ store.transport === 'wifi' ? 'Wi-Fi' : 'USB' }}</span>
            <span>&middot;</span>
            <span>{{ store.deviceId }}</span>
          </div>
          <!-- Line 2: model + battery -->
          <div class="flex items-center gap-6 flex-wrap">
            <div class="flex items-center gap-2">
              <Smartphone :size="14" class="text-accent-emerald shrink-0" />
              <span class="text-xs font-semibold text-theme-primary">{{ store.model }}</span>
              <span class="text-[10px] text-theme-muted">Android {{ store.androidVersion }}</span>
              <span class="text-[10px] text-theme-muted">SDK {{ store.sdkVersion }}</span>
              <span v-if="store.screenResolution" class="text-[10px] text-theme-muted">{{
                store.screenResolution
              }}</span>
            </div>

            <div class="h-4 w-px bg-theme-secondary"></div>

            <div class="flex items-center gap-2">
              <Battery
                :size="14"
                class="shrink-0"
                :class="store.batteryLevel > 20 ? 'text-color-success' : 'text-color-error'"
              />
              <span class="text-sm font-bold leading-none">{{ store.batteryLevel }}%</span>
              <span class="text-[10px]" :class="store.batteryColor">{{ store.batteryStatus }}</span>
              <template v-if="store.batteryTemp > 0 || store.batteryVoltage > 0">
                <span class="text-[10px] text-theme-muted">·</span>
                <Thermometer
                  v-if="store.batteryTemp > 0"
                  :size="11"
                  class="text-theme-muted shrink-0"
                />
                <span v-if="store.batteryTemp > 0" class="text-[10px] text-theme-muted"
                  >{{ store.batteryTemp.toFixed(1) }}°C</span
                >
                <span v-if="store.batteryVoltage > 0" class="text-[10px] text-theme-muted"
                  >{{ (store.batteryVoltage / 1000).toFixed(1) }}V</span
                >
              </template>
            </div>
          </div>

          <!-- Line 3: progress bars (RAM / Disk / CPU) — full width row -->
          <div
            v-if="store.ramTotal > 0 || store.storageTotal > 0"
            class="flex items-center gap-3 mt-1.5"
          >
            <div v-if="store.ramTotal > 0" class="flex items-center gap-1.5 flex-1 min-w-0">
              <MemoryStick :size="12" class="text-accent-emerald shrink-0" />
              <span class="text-[10px] text-theme-secondary w-7 shrink-0">RAM</span>
              <div class="flex-1 progress-bar-track min-w-0">
                <div class="progress-bar-fill" :style="{ width: store.ramPercent + '%' }"></div>
              </div>
              <span class="text-[10px] text-theme-secondary whitespace-nowrap"
                >{{ store.ramUsedGb }}/{{ store.ramTotalGb }} GB</span
              >
            </div>

            <div
              v-if="store.ramTotal > 0 && store.storageTotal > 0"
              class="h-4 w-px bg-theme-secondary shrink-0"
            ></div>

            <div v-if="store.storageTotal > 0" class="flex items-center gap-1.5 flex-1 min-w-0">
              <HardDrive :size="12" class="text-accent-emerald shrink-0" />
              <span class="text-[10px] text-theme-secondary w-7 shrink-0">Disk</span>
              <div class="flex-1 progress-bar-track min-w-0">
                <div class="progress-bar-fill" :style="{ width: store.storagePercent + '%' }"></div>
              </div>
              <span class="text-[10px] text-theme-secondary whitespace-nowrap"
                >{{ store.storageUsed }}/{{ store.storageTotal }} GB</span
              >
            </div>

            <div class="h-4 w-px bg-theme-secondary shrink-0"></div>

            <div class="flex items-center gap-1.5 flex-1 min-w-0">
              <Cpu :size="12" class="text-accent-emerald shrink-0" />
              <span class="text-[10px] text-theme-secondary w-7 shrink-0">CPU</span>
              <div class="flex-1 progress-bar-track min-w-0">
                <div class="progress-bar-fill" :style="{ width: store.cpuUsage + '%' }"></div>
              </div>
              <span class="text-[10px] text-theme-secondary whitespace-nowrap"
                >{{ store.cpuUsage.toFixed(0) }}%</span
              >
            </div>
          </div>

          <!-- Fallback: CPU only (when no RAM/Storage data) -->
          <div v-else class="flex items-center gap-2 mt-1.5">
            <Cpu :size="14" class="text-accent-emerald shrink-0" />
            <span class="text-[10px] text-theme-secondary w-7">CPU</span>
            <div class="flex-1 progress-bar-track">
              <div class="progress-bar-fill" :style="{ width: store.cpuUsage + '%' }"></div>
            </div>
            <span class="text-[10px] text-theme-secondary w-8 text-right"
              >{{ store.cpuUsage.toFixed(0) }}%</span
            >
          </div>

          <NetworkInfoRow :network="store.networkInfo" :loading="store.isLoadingStats" />
        </div>

        <!-- Connected badge + dropdown -->
        <div ref="badgeRef" class="shrink-0">
          <button
            class="btn-pressable status-badge status-badge-active hover:opacity-90 transition-opacity"
            @click="toggleDropdown"
          >
            <span class="status-dot status-dot-active"></span>
            <span>Connected</span>
            <ChevronDown
              :size="12"
              class="transition-transform duration-200"
              :class="dropdownOpen ? 'rotate-180' : ''"
            />
          </button>

          <Transition name="dropdown">
            <Teleport v-if="dropdownOpen" to="body">
              <div
                ref="dropdownRef"
                :style="dropdownStyle"
                class="fixed rounded-lg bg-theme-sidebar border border-theme-secondary shadow-theme-modal overflow-hidden z-[100]"
              >
                <button
                  class="btn-pressable w-full flex items-center gap-2 px-3 py-2.5 text-[11px] text-theme-secondary hover:bg-theme-hover hover:text-theme-primary transition-colors"
                  @click="handleReconnect"
                >
                  <RefreshCw :size="13" />
                  Reconnect
                </button>
                <button
                  class="btn-pressable w-full flex items-center gap-2 px-3 py-2.5 text-[11px] text-color-error hover:bg-color-error-container transition-colors border-t border-theme-tertiary"
                  @click="handleDisconnect"
                >
                  <Unplug :size="13" />
                  Disconnect
                </button>
              </div>
            </Teleport>
          </Transition>
        </div>
      </div>
    </template>

    <!-- Disconnected state -->
    <template v-else>
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-3 text-theme-muted">
          <Smartphone :size="16" class="opacity-30" />
          <span class="text-xs">No device connected</span>
        </div>
        <button
          class="btn-pressable btn-primary flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-semibold"
          @click="nav.openConnectPanel()"
        >
          <Zap :size="13" />
          Connect
        </button>
      </div>
    </template>
  </section>
</template>

<style scoped>
/* Dropdown motion (FR-8) — origin-aware scale + opacity, custom easing */
.dropdown-enter-active {
  transition:
    opacity var(--duration-standard) var(--ease-emphasized),
    transform var(--duration-standard) var(--ease-emphasized);
}
.dropdown-leave-active {
  transition:
    opacity 100ms var(--ease-accelerate),
    transform 100ms var(--ease-accelerate);
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: scale(0.97) translateY(-4px);
}
</style>
