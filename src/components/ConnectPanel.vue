<script setup lang="ts">
import { useDeviceStore } from '../stores/device';
import { useNavigationStore } from '../stores/navigation';
import { useConnectionHistoryStore } from '../stores/connectionHistory';
import { Link, Terminal, Bookmark, Check, Smartphone, X, Trash2, ChevronRight } from '@lucide/vue';
import ConnectAutoSection from './connect/ConnectAutoSection.vue';
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
        <ConnectAutoSection
          :connecting="store.connecting"
          :connected="store.connected"
          :status="store.autoConnectStatus"
          :device-model="store.model"
          :device-id="store.deviceId"
          :transport="store.transport"
          :error="store.autoConnectStatus === 'error'"
          error-message="No device found. Try USB or enter IP in Manual tab."
          @auto-connect="handleAutoConnect"
          @disconnect="store.disconnect()"
        />

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
