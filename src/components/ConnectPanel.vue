<script setup lang="ts">
import { useDeviceStore } from '../stores/device';
import { useNavigationStore } from '../stores/navigation';
import { useConnectionHistoryStore } from '../stores/connectionHistory';
import { Link, X } from '@lucide/vue';
import ConnectAutoSection from './connect/ConnectAutoSection.vue';
import QuickReconnectList from './connect/QuickReconnectList.vue';
import ManualConnectSection from './connect/ManualConnectSection.vue';
import { ref, computed, watch, nextTick } from 'vue';

const store = useDeviceStore();
const nav = useNavigationStore();
const history = useConnectionHistoryStore();

const savedDevices = computed(() => history.getAll());
const manualExpanded = ref(false);

// ── State machine refs (blueprint §7.3 v1.1) ──
const connectSource = ref<'auto' | 'saved' | 'manual' | null>(null);
const lastError = ref('');
let autoCloseTimer: ReturnType<typeof setTimeout> | null = null;

const showError = computed(() => store.autoConnectStatus === 'error' || lastError.value !== '');
const errorMessage = computed(() => {
  if (store.autoConnectStatus === 'error')
    return 'No device found. Try USB or enter IP in Manual tab.';
  return lastError.value;
});

// ── Auto-close watcher (FR-6) ──
watch(
  () => store.connected,
  (nowConnected, wasConnected) => {
    if (nowConnected && !wasConnected) {
      if (connectSource.value === 'auto' || connectSource.value === 'saved') {
        autoCloseTimer = setTimeout(() => {
          if (nav.connectPanelOpen) nav.closeConnectPanel();
          autoCloseTimer = null;
        }, 800);
      }
      connectSource.value = null;
    }
  }
);

// ── Cancel timer on manual close (FR-6 AC2) ──
watch(
  () => nav.connectPanelOpen,
  (open) => {
    if (!open && autoCloseTimer) {
      clearTimeout(autoCloseTimer);
      autoCloseTimer = null;
    }
  }
);

// ── Reset state on panel close (FR-4 AC5 + cleanup) ──
watch(
  () => nav.connectPanelOpen,
  (open) => {
    if (!open) {
      manualExpanded.value = false;
      connectSource.value = null;
      lastError.value = '';
    }
  }
);

// ── Auto-expand manual on error (FR-4 AC4) ──
watch(
  () => store.autoConnectStatus,
  (status) => {
    if (status === 'error') manualExpanded.value = true;
  }
);

// ── Action wrappers (blueprint §5) ──
async function onAutoConnect() {
  lastError.value = '';
  connectSource.value = 'auto';
  await store.autoConnect();
  // autoConnect() sets autoConnectStatus='error' on failure (caught by showError computed)
  // If connected succeeded, auto-close watcher already consumed connectSource during onConnected()
  connectSource.value = null;
}

async function onConnectSaved(device: { ip: string; port: number; method: 'wifi' | 'pairing' }) {
  lastError.value = '';
  connectSource.value = 'saved';
  await store.connectSaved(device);
  // store.connectWithPort() catches errors internally (toast only, no throw)
  // Detect failure by checking connected state after await
  if (!store.connected) {
    lastError.value = 'Connection failed. Check the device is reachable.';
  }
  connectSource.value = null;
}

async function onManualConnect(ip: string, port: number) {
  lastError.value = '';
  connectSource.value = 'manual';
  await store.connectWithPort(ip, port);
  // store.connectWithPort() catches errors internally (toast only, no throw)
  // Detect failure by checking connected state after await
  if (!store.connected) {
    lastError.value = 'Connection failed. Check the IP and port.';
  }
  connectSource.value = null;
}

// ── Focus trap (blueprint §4 v1.1) ──
const panelRef = ref<HTMLElement | null>(null);
const FOCUSABLE =
  'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';
let triggerElement: Element | null = null;

function onPanelKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    nav.closeConnectPanel();
    return;
  }
  if (e.key !== 'Tab') return;
  const el = panelRef.value;
  if (!el) return;
  const focusable = el.querySelectorAll(FOCUSABLE);
  if (focusable.length === 0) return;
  const first = focusable[0] as HTMLElement;
  const last = focusable[focusable.length - 1] as HTMLElement;
  if (e.shiftKey && document.activeElement === first) {
    e.preventDefault();
    last.focus();
  } else if (!e.shiftKey && document.activeElement === last) {
    e.preventDefault();
    first.focus();
  }
}

watch(
  () => nav.connectPanelOpen,
  (open) => {
    if (open) {
      triggerElement = document.activeElement;
      nextTick(() => {
        const el = panelRef.value;
        if (el) {
          const first = el.querySelector<HTMLElement>(FOCUSABLE);
          first?.focus();
        }
      });
    } else {
      if (triggerElement instanceof HTMLElement) triggerElement.focus();
      triggerElement = null;
    }
  }
);

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
      ref="panelRef"
      role="dialog"
      aria-label="Connect to device"
      aria-modal="true"
      class="fixed top-[36px] right-0 h-[calc(100dvh-36px)] w-[340px] z-40 flex flex-col bg-theme-sidebar border-l border-theme-tertiary shadow-theme-modal"
      @keydown="onPanelKeydown"
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
          aria-label="Close panel"
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
          :error="showError"
          :error-message="errorMessage"
          @auto-connect="onAutoConnect"
          @disconnect="store.disconnect()"
        />

        <!-- Divider -->
        <div class="border-t border-theme-tertiary my-3"></div>

        <!-- 2. Quick Reconnect Section -->
        <QuickReconnectList
          :devices="savedDevices"
          :current-device-id="store.deviceId || ''"
          :connecting="store.connecting"
          @connect="onConnectSaved"
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
              onManualConnect(ip, port);
            }
          "
          @update:expanded="manualExpanded = $event"
        />
      </div>
    </div>
  </Transition>
</template>
