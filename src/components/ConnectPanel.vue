<script setup lang="ts">
import { useDeviceStore } from '../stores/device';
import { useNavigationStore } from '../stores/navigation';
import { useConnectionHistoryStore } from '../stores/connectionHistory';
import { Link, X } from '@lucide/vue';
import ConnectAutoSection from './connect/ConnectAutoSection.vue';
import QuickReconnectList from './connect/QuickReconnectList.vue';
import ManualConnectSection from './connect/ManualConnectSection.vue';
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
        <QuickReconnectList
          :devices="savedDevices"
          :current-device-id="store.deviceId || ''"
          :connecting="store.connecting"
          @connect="handleConnectSaved"
          @forget="history.remove"
        />

        <!-- Divider -->
        <div class="border-t border-theme-tertiary my-3"></div>

        <!-- 3. Manual Connect Section (collapsible) -->
        <ManualConnectSection
          :connecting="store.connecting"
          :expanded="manualExpanded"
          @connect="
            (ip, port) => {
              store.ipAddress = ip;
              store.port = port;
              handleConnect();
            }
          "
          @update:expanded="manualExpanded = $event"
        />
      </div>
    </div>
  </Transition>
</template>
