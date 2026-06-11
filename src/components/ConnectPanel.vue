<script setup lang="ts">
import { useDeviceStore } from '../stores/device';
import { useNavigationStore } from '../stores/navigation';
import { useConnectionHistoryStore } from '../stores/connectionHistory';
import {
  Link,
  Zap,
  Terminal,
  Bookmark,
  Check,
  Smartphone,
  XCircle,
  Loader2,
  X,
  Trash2,
  ChevronRight,
} from '@lucide/vue';
import { ref, computed, watch } from 'vue';

const store = useDeviceStore();
const nav = useNavigationStore();
const history = useConnectionHistoryStore();

const savedDevices = computed(() => history.getAll());
const manualExpanded = ref(false);

let wasConnecting = false;
watch(
  () => store.connecting,
  (val) => {
    wasConnecting = val || wasConnecting;
  }
);
watch(
  () => store.connected,
  (val) => {
    if (val && wasConnecting && nav.connectPanelOpen) {
      wasConnecting = false;
      nav.closeConnectPanel();
    }
  }
);

function handleConnect() {
  if (store.ipAddress.trim()) {
    store.connectWithRetry();
  } else {
    store.autoConnect();
  }
}

async function handleAutoConnect() {
  await store.autoConnect();
}

async function handleConnectSaved(device: {
  ip: string;
  port: number;
  method: 'wifi' | 'pairing';
}) {
  await store.connectSaved(device);
  if (store.connected) {
    nav.closeConnectPanel();
  }
}

function forgetDevice(id: string) {
  history.remove(id);
}

const statusLabel = computed(() => {
  switch (store.autoConnectStatus) {
    case 'idle':
      return '';
    case 'detecting_usb':
      return 'Detecting USB devices...';
    case 'enabling_tcp':
      return 'Enabling wireless mode...';
    case 'detecting_ip':
      return 'Detecting IP address...';
    case 'connecting_tcp':
      return 'Connecting via Wi-Fi...';
    case 'connected':
      return '';
    case 'error':
      return '';
    default:
      return '';
  }
});

function closePanel() {
  nav.closeConnectPanel();
}

function handleBackdropClick() {
  closePanel();
}
</script>

<template>
  <!-- Backdrop -->
  <Transition name="slide-panel-backdrop">
    <div
      v-if="nav.connectPanelOpen"
      class="fixed inset-0 top-[36px] z-30 bg-black/50 backdrop-blur-sm"
      @click="handleBackdropClick"
    ></div>
  </Transition>

  <!-- Panel -->
  <Transition name="slide-panel">
    <div
      v-if="nav.connectPanelOpen"
      class="fixed top-[36px] right-0 h-[calc(100dvh-36px)] w-[340px] z-40 flex flex-col bg-theme-sidebar border-l border-theme-tertiary shadow-theme-modal"
    >
      <!-- Fixed Header -->
      <div
        class="flex items-center justify-between px-5 py-4 border-b border-theme-tertiary shrink-0"
      >
        <div class="flex items-center gap-2.5">
          <div
            class="w-8 h-8 rounded-lg bg-accent-light border border-accent-default flex items-center justify-center shrink-0"
          >
            <Link :size="15" class="text-accent-emerald" />
          </div>
          <div>
            <h2 class="font-sans text-sm font-semibold tracking-wider uppercase leading-tight">
              Connect
            </h2>
            <p class="text-[10px] text-theme-muted leading-tight mt-0.5">Connect to your device</p>
          </div>
        </div>
        <button
          class="p-1.5 rounded-lg text-theme-muted hover:text-theme-primary hover:bg-theme-hover transition-colors"
          @click="closePanel"
        >
          <X :size="16" />
        </button>
      </div>

      <!-- Scrollable Body -->
      <div class="flex-1 overflow-y-auto px-5 pb-6">
        <!-- 1. Auto Connect Section -->
        <div class="space-y-4 pt-2">
          <div class="text-center py-6">
            <div
              class="w-12 h-12 rounded-2xl bg-accent-light border border-accent-default flex items-center justify-center mx-auto mb-3"
            >
              <Zap :size="24" class="text-accent-emerald" />
            </div>
            <p class="text-xs text-theme-secondary mb-4">
              Plug in USB or connect to an existing device
            </p>
            <button
              :disabled="store.connecting || store.autoConnectStatus === 'connected'"
              class="w-full btn-primary py-3 rounded-lg text-sm font-semibold disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
              @click="handleAutoConnect"
            >
              <Zap :size="16" />
              <span>{{ store.connecting ? 'Connecting...' : 'Auto Connect' }}</span>
            </button>
          </div>

          <!-- Connecting progress bar -->
          <div
            v-if="store.connecting"
            class="flex items-center gap-2 py-2 px-3 rounded-md bg-accent-light text-[11px] text-accent-emerald"
          >
            <Loader2 :size="14" class="animate-spin shrink-0" />
            <span>{{ statusLabel || 'Connecting...' }}</span>
          </div>

          <!-- Connected state -->
          <div v-if="store.autoConnectStatus === 'connected' && store.connected" class="space-y-3">
            <div
              class="flex items-center gap-2 py-2.5 px-3 rounded-lg bg-accent-light border border-accent-default"
            >
              <Check :size="14" class="text-accent-emerald shrink-0" />
              <div>
                <div class="text-xs font-semibold text-accent-emerald">
                  {{ store.model || store.deviceId }}
                </div>
                <div class="text-[10px] text-theme-muted mt-0.5">
                  {{ store.transport === 'wifi' ? 'Wi-Fi' : 'USB' }}
                  <span class="mx-0.5">&middot;</span>
                  {{ store.deviceId }}
                </div>
              </div>
            </div>
            <div v-if="store.transport === 'wifi'" class="text-[10px] text-theme-muted px-1">
              You may unplug USB safely
            </div>
            <div v-else class="text-[10px] text-theme-muted px-1">
              Unplugging USB will disconnect the device
            </div>
          </div>

          <!-- Error state -->
          <div
            v-if="store.autoConnectStatus === 'error'"
            class="flex items-center gap-2 py-2.5 px-3 rounded-lg bg-color-error-container border border-color-error"
          >
            <XCircle :size="14" class="text-color-error shrink-0" />
            <span class="text-[11px] text-color-error"
              >No device found. Try USB or enter IP in Manual tab.</span
            >
          </div>
        </div>

        <!-- Divider -->
        <div class="border-t border-theme-tertiary my-3"></div>

        <!-- 2. Quick Reconnect Section -->
        <div class="space-y-2.5">
          <div v-if="savedDevices.length === 0" class="text-center py-10">
            <Bookmark :size="28" class="mx-auto mb-2 opacity-20 text-theme-muted" />
            <p class="text-xs text-theme-muted">No saved devices yet</p>
            <p class="text-[10px] text-theme-muted mt-1">
              Devices will appear here after connecting
            </p>
          </div>
          <div
            v-for="device in savedDevices"
            :key="device.id"
            class="flex items-start gap-3 py-3 px-3 rounded-lg border-l-4 border-accent-emerald bg-theme-hover/20 hover:bg-theme-hover/50 transition-colors"
          >
            <Smartphone :size="16" class="text-accent-emerald shrink-0 mt-0.5" />
            <div class="flex-1 min-w-0">
              <div class="text-xs font-semibold text-theme-primary truncate">
                {{ device.label }}
              </div>
              <div class="text-[10px] text-theme-muted mt-0.5">
                {{ device.ip }}:{{ device.port }}
                <span class="mx-1">&middot;</span>
                {{ device.method }}
              </div>
              <div class="text-[10px] text-theme-muted">
                Last: {{ new Date(device.lastConnected).toLocaleDateString() }}
              </div>
            </div>
            <div class="flex items-center gap-1.5 shrink-0">
              <button
                class="btn-primary text-[10px] py-1 px-2.5 rounded-md font-semibold disabled:opacity-50"
                :disabled="store.connected && store.deviceId === `${device.ip}:${device.port}`"
                @click="handleConnectSaved(device)"
              >
                Connect
              </button>
              <button
                class="p-1.5 rounded-md text-theme-muted hover:text-color-error hover:bg-color-error-container transition-colors"
                title="Forget device"
                @click="forgetDevice(device.id)"
              >
                <Trash2 :size="13" />
              </button>
            </div>
          </div>
        </div>

        <!-- Divider -->
        <div class="border-t border-theme-tertiary my-3"></div>

        <!-- 3. Manual Connect Section (collapsible) -->
        <div>
          <button
            class="w-full flex items-center justify-between py-2 px-1 text-xs font-medium text-theme-secondary hover:text-theme-primary hover:bg-theme-hover/50 rounded-lg transition-colors"
            @click="manualExpanded = !manualExpanded"
          >
            <span>Connect via IP address</span>
            <ChevronRight
              :size="14"
              class="transition-transform duration-200"
              :class="{ 'rotate-90': manualExpanded }"
            />
          </button>
          <div v-if="manualExpanded" class="space-y-4 pt-2">
            <div class="space-y-3">
              <div>
                <label
                  class="text-[10px] text-theme-muted uppercase tracking-wider font-semibold mb-1.5 block"
                  >IP Address</label
                >
                <input
                  v-model="store.ipAddress"
                  type="text"
                  placeholder="e.g., 192.168.1.5"
                  class="w-full input-terminal py-2 px-3 text-xs text-theme-primary placeholder:text-theme-muted"
                />
              </div>
              <div>
                <label
                  class="text-[10px] text-theme-muted uppercase tracking-wider font-semibold mb-1.5 block"
                  >Port</label
                >
                <input
                  v-model.number="store.port"
                  type="number"
                  placeholder="5555"
                  class="w-full input-terminal py-2 px-3 text-xs text-theme-primary placeholder:text-theme-muted"
                />
              </div>
            </div>
            <div class="flex gap-2">
              <button
                :disabled="store.connecting || !store.ipAddress.trim()"
                class="flex-1 btn-primary py-2.5 rounded-lg text-xs font-semibold disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-1.5"
                @click="handleConnect"
              >
                <Terminal :size="13" />
                {{ store.connecting ? 'Connecting...' : 'Connect' }}
              </button>
              <button
                v-if="store.connecting"
                class="bg-theme-btn border border-color-error text-color-error py-2.5 px-3 rounded-lg text-xs hover-accent transition-colors"
                @click="store.cancelConnect()"
              >
                Cancel
              </button>
            </div>

            <!-- Connected state in manual section -->
            <div v-if="store.connected" class="space-y-3 pt-2">
              <div
                class="flex items-center gap-2 py-2.5 px-3 rounded-lg bg-accent-light border border-accent-default"
              >
                <Check :size="14" class="text-accent-emerald shrink-0" />
                <div class="flex-1 min-w-0">
                  <div class="text-xs font-semibold text-accent-emerald truncate">
                    {{ store.model || store.deviceId }}
                  </div>
                  <div class="text-[10px] text-theme-muted">
                    {{ store.transport === 'wifi' ? 'Wi-Fi' : 'USB' }} &middot; {{ store.deviceId }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>
