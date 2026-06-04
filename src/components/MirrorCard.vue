<script setup lang="ts">
import { useDeviceStore } from '../stores/device';
import { useSettingsStore } from '../stores/settings';
import { useNavigationStore } from '../stores/navigation';
import {
  MonitorPlay,
  Settings,
  Square,
  Sun,
  RotateCw,
  AlertTriangle,
  Play,
  ChevronDown,
  Eye,
  MonitorOff,
  PinOff,
  MousePointerClick,
} from '@lucide/vue';
import { computed, ref } from 'vue';

const store = useDeviceStore();
const settings = useSettingsStore();
const nav = useNavigationStore();

const expanded = ref(true);

const showScrcpyWarning = computed(() => !settings.scrcpyValid && !store.scrcpyAvailable);

const statusState = computed(() => {
  if (store.mirroring && store.recordingScreen) return 'recording';
  if (store.mirroring) return 'active';
  return 'idle';
});

const statusLabel = computed(() => {
  switch (statusState.value) {
    case 'recording':
      return 'Recording';
    case 'active':
      return 'Mirroring';
    default:
      return 'Idle';
  }
});

// Mutual exclusivity rules
const noControlDisabled = computed(() => store.stayAwakeEnabled);
const screenOffDisabled = computed(() => store.noControl);

function toggleMirror() {
  if (store.mirroring) {
    store.stopMirror();
  } else {
    store.launchMirror();
  }
}
</script>

<template>
  <section class="card-glass p-4">
    <!-- Header Toolbar -->
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <MonitorPlay :size="16" class="text-accent-emerald" />
        <h2 class="font-sans text-xs font-semibold tracking-wider uppercase">Mirror</h2>
      </div>
      <div class="flex items-center gap-2">
        <!-- Status Badge -->
        <div
          class="status-badge"
          :class="{
            'status-badge-idle': statusState === 'idle',
            'status-badge-active': statusState === 'active',
            'status-badge-recording': statusState === 'recording',
          }"
        >
          <span
            class="status-dot"
            :class="{
              'status-dot-idle': statusState === 'idle',
              'status-dot-active': statusState === 'active',
              'status-dot-recording': statusState === 'recording',
            }"
          ></span>
          {{ statusLabel }}
        </div>

        <!-- Record Toggle Pill -->
        <div
          class="toggle-pill"
          :class="{
            'toggle-pill-on': store.recordingScreen,
            'opacity-40 cursor-not-allowed': !store.connected,
          }"
          @click="store.connected && store.toggleRecording()"
        >
          <span class="toggle-pill-knob"></span>
          <span>{{ store.recordingScreen ? 'Recording' : 'Record' }}</span>
        </div>

        <!-- Start/Stop Button (Compact) -->
        <button
          :disabled="!store.connected"
          class="btn-pressable text-xs px-3 py-1.5 rounded-lg font-semibold transition-all duration-200 disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1.5"
          :class="
            store.mirroring
              ? 'bg-accent-emerald text-theme-inverse border border-accent-emerald hover:bg-accent-emerald-hover'
              : 'btn-primary border border-theme-secondary hover:border-accent-emerald'
          "
          @click="toggleMirror"
        >
          <template v-if="store.mirroring">
            <Square :size="14" />
            <span>Stop</span>
          </template>
          <template v-else>
            <Play :size="14" />
            <span>Start</span>
          </template>
        </button>
      </div>
    </div>

    <!-- scrcpy Warning -->
    <div
      v-if="showScrcpyWarning"
      class="flex items-start gap-2 text-[11px] text-color-warning bg-color-warning-container border border-color-warning rounded-lg p-2.5 mb-3"
    >
      <AlertTriangle :size="14" class="shrink-0 mt-0.5" />
      <div class="flex-1">
        <template v-if="settings.scrcpyPath && !settings.scrcpyValid">
          Configured scrcpy path is invalid.
        </template>
        <template v-else> scrcpy not found. Install it or set the path in Settings. </template>
        <button
          class="ml-1 underline hover:text-color-warning inline-flex items-center gap-1"
          @click="nav.navigateTo('settings')"
        >
          <Settings :size="10" /> Settings
        </button>
      </div>
    </div>

    <!-- Quick Actions Header (Expandable) -->
    <button
      class="flex items-center gap-1.5 text-[10px] text-theme-muted uppercase tracking-wider font-semibold mb-2 w-full hover:text-theme-secondary transition-colors"
      @click="expanded = !expanded"
    >
      <ChevronDown
        :size="12"
        class="transition-transform duration-200"
        :class="{ '-rotate-90': !expanded }"
      />
      Quick Actions
    </button>

    <!-- Quick Actions Grid -->
    <div v-show="expanded" class="grid grid-cols-3 gap-2">
      <button
        class="action-card btn-pressable"
        :class="{ 'action-card-active': store.showTouches }"
        @click="store.toggleMirrorFlag('showTouches')"
      >
        <MousePointerClick :size="14" />
        <span class="flex-1 text-left text-[11px]">Show Touches</span>
        <span v-if="store.showTouches" class="text-[9px] uppercase font-bold opacity-70">ON</span>
      </button>

      <button
        class="action-card btn-pressable"
        :disabled="screenOffDisabled"
        :class="{
          'action-card-active': store.turnScreenOff,
          'opacity-40 cursor-not-allowed': screenOffDisabled,
        }"
        :title="screenOffDisabled ? 'Cannot turn screen off when View Only is active' : ''"
        @click="store.toggleMirrorFlag('turnScreenOff')"
      >
        <MonitorOff :size="14" />
        <span class="flex-1 text-left text-[11px]">Screen Off</span>
        <span v-if="store.turnScreenOff" class="text-[9px] uppercase font-bold opacity-70">ON</span>
      </button>

      <button
        class="action-card btn-pressable"
        :class="{ 'action-card-active': store.alwaysOnTop }"
        @click="store.toggleMirrorFlag('alwaysOnTop')"
      >
        <PinOff :size="14" />
        <span class="flex-1 text-left text-[11px]">Always Top</span>
        <span v-if="store.alwaysOnTop" class="text-[9px] uppercase font-bold opacity-70">ON</span>
      </button>

      <button
        class="action-card btn-pressable"
        :disabled="noControlDisabled"
        :class="{
          'action-card-active': store.noControl,
          'opacity-40 cursor-not-allowed': noControlDisabled,
        }"
        :title="noControlDisabled ? 'Cannot disable control when Stay Awake is active' : ''"
        @click="store.toggleMirrorFlag('noControl')"
      >
        <Eye :size="14" />
        <span class="flex-1 text-left text-[11px]">View Only</span>
        <span v-if="store.noControl" class="text-[9px] uppercase font-bold opacity-70">ON</span>
      </button>

      <button
        class="action-card btn-pressable"
        :class="{ 'action-card-active': store.stayAwakeEnabled }"
        @click="store.toggleStayAwake(!store.stayAwakeEnabled)"
      >
        <Sun :size="14" />
        <span class="flex-1 text-left text-[11px]">Stay Awake</span>
        <span v-if="store.stayAwakeEnabled" class="text-[9px] uppercase font-bold opacity-70"
          >ON</span
        >
      </button>

      <button class="action-card btn-pressable" @click="store.rotateDevice()">
        <RotateCw :size="14" />
        <span class="flex-1 text-left text-[11px]">Rotate</span>
      </button>
    </div>
  </section>
</template>
