<script setup lang="ts">
import { useDeviceStore } from '../stores/device';
import { useNavigationStore } from '../stores/navigation';
import { Smartphone, Battery, Cpu, Zap, ChevronDown, RefreshCw, Unplug } from '@lucide/vue';
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';

const store = useDeviceStore();
const nav = useNavigationStore();
const dropdownOpen = ref(false);
const dropdownRef = ref<HTMLElement | null>(null);
const dropdownStyle = ref<Record<string, string>>({});
const badgeRef = ref<HTMLElement | null>(null);

function toggleDropdown() {
  dropdownOpen.value = !dropdownOpen.value;
  if (dropdownOpen.value) {
    nextTick(updateDropdownPosition);
  }
}

function updateDropdownPosition() {
  if (badgeRef.value) {
    const rect = badgeRef.value.getBoundingClientRect();
    dropdownStyle.value = {
      position: 'fixed',
      top: `${rect.bottom + 4}px`,
      left: `${rect.left}px`,
      minWidth: '160px',
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
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
  window.removeEventListener('scroll', handleScroll, true);
});

watch(dropdownOpen, (val) => {
  if (val) {
    nextTick(updateDropdownPosition);
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
          <div class="h-2.5 bg-gray-700 rounded-full animate-pulse w-full max-w-[200px]"></div>
          <div class="h-2.5 bg-gray-600 rounded-full animate-pulse w-full max-w-[320px]"></div>
        </div>
      </div>
    </template>

    <!-- Connected state -->
    <template v-else-if="store.connected">
      <div class="flex items-center justify-between gap-4">
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 text-[10px] text-theme-muted mb-1">
            <span class="capitalize">{{
              store.connectMethod === 'pairing' ? 'Android 11+ Pairing' : store.connectMethod
            }}</span>
            <span>&middot;</span>
            <span>{{ store.transport === 'wifi' ? 'Wi-Fi' : 'USB' }}</span>
            <span>&middot;</span>
            <span>{{ store.deviceId }}</span>
          </div>
          <div class="flex items-center gap-6 flex-wrap">
            <div class="flex items-center gap-2">
              <Smartphone :size="14" class="text-accent-emerald shrink-0" />
              <span class="text-xs font-semibold text-theme-primary">{{ store.model }}</span>
              <span class="text-[10px] text-theme-muted">Android {{ store.androidVersion }}</span>
              <span class="text-[10px] text-theme-muted">SDK {{ store.sdkVersion }}</span>
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
            </div>

            <div class="h-4 w-px bg-theme-secondary"></div>

            <div class="flex items-center gap-2 min-w-[140px]">
              <Cpu :size="14" class="text-accent-emerald shrink-0" />
              <span class="text-[10px] text-theme-secondary w-7">CPU</span>
              <div class="flex-1 progress-bar-track">
                <div
                  class="progress-bar-fill transition-all duration-500"
                  :style="{ width: store.cpuUsage + '%' }"
                ></div>
              </div>
              <span class="text-[10px] text-theme-secondary w-8 text-right"
                >{{ store.cpuUsage.toFixed(0) }}%</span
              >
            </div>
          </div>
        </div>

        <!-- Connected badge + dropdown -->
        <div ref="badgeRef" class="shrink-0">
          <button
            class="status-badge status-badge-active hover:opacity-90 transition-opacity"
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
                  class="w-full flex items-center gap-2 px-3 py-2.5 text-[11px] text-theme-secondary hover:bg-theme-hover hover:text-theme-primary transition-colors"
                  @click="handleReconnect"
                >
                  <RefreshCw :size="13" />
                  Reconnect
                </button>
                <button
                  class="w-full flex items-center gap-2 px-3 py-2.5 text-[11px] text-color-error hover:bg-color-error-container transition-colors border-t border-theme-tertiary"
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
          class="btn-primary flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-xs font-semibold"
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
.dropdown-enter-active {
  transition: all 0.15s ease-out;
}
.dropdown-leave-active {
  transition: all 0.1s ease-in;
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
