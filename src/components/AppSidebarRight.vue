<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { useDeviceStore } from '../stores/device';
import {
  Wifi,
  Plane,
  Bluetooth,
  Pointer,
  LayoutGrid,
  Sun,
  Scaling,
  Home,
  ArrowLeft,
  History,
  Volume1,
  Volume2,
  VolumeX,
  Power,
  SkipBack,
  Play,
  SkipForward,
  Send,
  Image,
  PanelRightClose,
  PanelRightOpen,
  ChevronDown,
  RotateCcw,
  MonitorUp,
  Smartphone,
} from '@lucide/vue';

const store = useDeviceStore();
const collapsed = ref(false);

/* FR-7: lifecycle for is-resizing class (R3, P26) */
const sidebarRef = ref<HTMLElement | null>(null);
const resizingTimeoutId = ref<number | null>(null);

/* FR-9: active section indicator when collapsed.
   IntersectionObserver tracks which section is most visible,
   highlights its collapsed icon with text-accent-emerald. */
type SectionId = 'connectivity' | 'devTools' | 'display' | 'remote' | 'system';
const activeSection = ref<SectionId | null>(null);
let observer: IntersectionObserver | null = null;

function toggleSidebar() {
  collapsed.value = !collapsed.value;
  sidebarRef.value?.classList.add('is-resizing');
  if (resizingTimeoutId.value !== null) {
    clearTimeout(resizingTimeoutId.value);
  }
  resizingTimeoutId.value = window.setTimeout(() => {
    sidebarRef.value?.classList.remove('is-resizing');
    resizingTimeoutId.value = null;
  }, 250);
}

function onTransitionEnd(e: TransitionEvent) {
  if (e.propertyName === 'width') {
    sidebarRef.value?.classList.remove('is-resizing');
    if (resizingTimeoutId.value !== null) {
      clearTimeout(resizingTimeoutId.value);
      resizingTimeoutId.value = null;
    }
  }
}

/* FR-9: setup IntersectionObserver for active section tracking.
   Only active when sidebar is collapsed. Threshold 0.3 means
   section must be 30% visible to count. */
function setupObserver() {
  if (observer) return; // idempotent
  const refs: Array<{ el: HTMLElement | null; id: SectionId }> = [
    { el: connectivityRef.value, id: 'connectivity' },
    { el: devToolsRef.value, id: 'devTools' },
    { el: displayRef.value, id: 'display' },
    { el: remoteRef.value, id: 'remote' },
    { el: systemRef.value, id: 'system' },
  ];
  const targets = refs.filter((r) => r.el).map((r) => r.el!);
  if (targets.length === 0) return;

  observer = new IntersectionObserver(
    (entries) => {
      const visible = entries
        .filter((e) => e.isIntersecting)
        .sort((a, b) => b.intersectionRatio - a.intersectionRatio);
      if (visible.length > 0) {
        const id = (visible[0].target as HTMLElement).dataset.section as SectionId;
        activeSection.value = id;
      } else {
        activeSection.value = null;
      }
    },
    { threshold: 0.3, root: sidebarRef.value }
  );
  targets.forEach((el) => observer?.observe(el));
}

function teardownObserver() {
  observer?.disconnect();
  observer = null;
  activeSection.value = null;
}

const connectivityRef = ref<HTMLElement | null>(null);
const devToolsRef = ref<HTMLElement | null>(null);
const displayRef = ref<HTMLElement | null>(null);
const remoteRef = ref<HTMLElement | null>(null);
const systemRef = ref<HTMLElement | null>(null);

function expandAndScroll(refVal: HTMLElement | null) {
  collapsed.value = false;
  nextTick(() => {
    refVal?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  });
}

function handleClickOutside(e: MouseEvent) {
  if (store.showRebootMenu) {
    const target = e.target as HTMLElement;
    if (!target.closest('.reboot-menu-container')) {
      store.showRebootMenu = false;
    }
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  if (resizingTimeoutId.value !== null) {
    clearTimeout(resizingTimeoutId.value);
  }
  teardownObserver();
});

/* FR-9: watch collapsed state — setup observer when collapsed=true,
   tear down when expanded. Sidebar must be in DOM with width 12 (collapsed)
   for IntersectionObserver to work. */
watch(
  collapsed,
  (isCollapsed) => {
    if (isCollapsed) {
      nextTick(() => {
        // Wait for sidebar width transition to complete
        setTimeout(setupObserver, 250);
      });
    } else {
      teardownObserver();
    }
  },
  { immediate: true }
);
</script>

<template>
  <aside
    ref="sidebarRef"
    class="bg-sidebar-dark sidebar-blur border-l border-theme-tertiary flex flex-col shrink-0 overflow-hidden z-20"
    :class="collapsed ? 'w-12' : 'w-48'"
    :style="{ transition: 'width var(--duration-standard) var(--ease-out)' }"
    @transitionend="onTransitionEnd"
  >
    <div
      class="flex items-center shrink-0"
      :class="collapsed ? 'justify-center p-3' : 'justify-end p-2'"
    >
      <button
        class="btn-pressable p-1.5 rounded-lg hover:bg-theme-hover transition-all"
        :title="collapsed ? 'Expand sidebar' : 'Collapse sidebar'"
        @click="toggleSidebar"
      >
        <PanelRightOpen v-if="collapsed" :size="16" class="opacity-70" />
        <PanelRightClose v-else :size="16" class="opacity-70" />
      </button>
    </div>

    <!-- Collapsed: section icons -->
    <div v-if="collapsed" class="flex flex-col items-center gap-1 px-2 mt-1">
      <button
        class="btn-pressable p-2.5 rounded-xl transition-all"
        :class="[
          activeSection === 'connectivity'
            ? 'text-accent-emerald bg-accent-emerald/10'
            : 'hover:bg-accent-emerald',
        ]"
        title="Connectivity"
        @click="expandAndScroll(connectivityRef)"
      >
        <Wifi :size="16" :class="activeSection === 'connectivity' ? '' : 'opacity-70'" />
      </button>
      <button
        class="btn-pressable p-2.5 rounded-xl transition-all"
        :class="[
          activeSection === 'devTools'
            ? 'text-accent-emerald bg-accent-emerald/10'
            : 'hover:bg-accent-emerald',
        ]"
        title="Dev Tools"
        @click="expandAndScroll(devToolsRef)"
      >
        <Pointer :size="16" :class="activeSection === 'devTools' ? '' : 'opacity-70'" />
      </button>
      <button
        class="btn-pressable p-2.5 rounded-xl transition-all"
        :class="[
          activeSection === 'display'
            ? 'text-accent-emerald bg-accent-emerald/10'
            : 'hover:bg-accent-emerald',
        ]"
        title="Display"
        @click="expandAndScroll(displayRef)"
      >
        <Sun :size="16" :class="activeSection === 'display' ? '' : 'opacity-70'" />
      </button>
      <button
        class="btn-pressable p-2.5 rounded-xl transition-all"
        :class="[
          activeSection === 'remote'
            ? 'text-accent-emerald bg-accent-emerald/10'
            : 'hover:bg-accent-emerald',
        ]"
        title="Remote Controls"
        @click="expandAndScroll(remoteRef)"
      >
        <Smartphone :size="16" :class="activeSection === 'remote' ? '' : 'opacity-70'" />
      </button>
      <button
        class="btn-pressable p-2.5 rounded-xl transition-all"
        :class="[
          activeSection === 'system'
            ? 'text-accent-emerald bg-accent-emerald/10'
            : 'hover:bg-accent-emerald',
        ]"
        title="System"
        @click="expandAndScroll(systemRef)"
      >
        <Image :size="16" :class="activeSection === 'system' ? '' : 'opacity-70'" />
      </button>
    </div>

    <!-- Expanded: full content -->
    <div v-if="!collapsed" class="flex flex-col p-3 overflow-y-auto">
      <!-- Connectivity -->
      <div ref="connectivityRef" data-section="connectivity" class="mb-6">
        <h3 class="font-sans text-xs font-semibold tracking-wider mb-4 uppercase text-theme-muted">
          Connectivity
        </h3>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Wifi :size="14" class="opacity-70" />
              <span class="text-xs text-theme-primary">Wi-Fi</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer select-none">
              <input
                v-model="store.wifiEnabled"
                type="checkbox"
                class="sr-only peer"
                @change="store.toggleWifi(store.wifiEnabled)"
              />
              <div
                class="w-8 h-[18px] bg-theme-toggle-track rounded-full peer peer-focus:outline-none after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[#06100d] after:rounded-full after:h-3.5 after:w-3.5 after:shadow-md after:transition-all after:duration-300 after:ease-[cubic-bezier(0.34,1.56,0.64,1)] peer-checked:after:translate-x-[12px] rtl:peer-checked:after:-translate-x-[12px] peer-checked:after:bg-accent-emerald peer-checked:after:bg-accent-emerald peer-active:after:w-[18px]"
              ></div>
            </label>
          </div>

          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Plane :size="14" class="opacity-70" />
              <span class="text-xs text-theme-primary">Airplane</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer select-none">
              <input
                v-model="store.airplaneEnabled"
                type="checkbox"
                class="sr-only peer"
                @change="store.toggleAirplane(store.airplaneEnabled)"
              />
              <div
                class="w-8 h-[18px] bg-theme-toggle-track rounded-full peer peer-focus:outline-none peer-checked:bg-accent-emerald after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[#06100d] after:rounded-full after:h-3.5 after:w-3.5 after:shadow-md after:transition-all after:duration-300 after:ease-[cubic-bezier(0.34,1.56,0.64,1)] peer-checked:after:translate-x-[12px] rtl:peer-checked:after:-translate-x-[12px] peer-checked:after:bg-accent-emerald peer-checked:after:bg-accent-emerald peer-active:after:w-[18px]"
              ></div>
            </label>
          </div>

          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Bluetooth :size="14" class="opacity-70" />
              <span class="text-xs text-theme-primary">Bluetooth</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer select-none">
              <input
                v-model="store.bluetoothEnabled"
                type="checkbox"
                class="sr-only peer"
                @change="store.toggleBluetooth(store.bluetoothEnabled)"
              />
              <div
                class="w-8 h-[18px] bg-theme-toggle-track rounded-full peer peer-focus:outline-none peer-checked:bg-accent-emerald after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[#06100d] after:rounded-full after:h-3.5 after:w-3.5 after:shadow-md after:transition-all after:duration-300 after:ease-[cubic-bezier(0.34,1.56,0.64,1)] peer-checked:after:translate-x-[12px] rtl:peer-checked:after:-translate-x-[12px] peer-checked:after:bg-accent-emerald peer-checked:after:bg-accent-emerald peer-active:after:w-[18px]"
              ></div>
            </label>
          </div>
        </div>
      </div>

      <!-- Dev Tools -->
      <div ref="devToolsRef" data-section="devTools" class="mb-6">
        <h3 class="font-sans text-xs font-semibold tracking-wider mb-4 uppercase text-theme-muted">
          Dev Tools
        </h3>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Pointer :size="14" class="opacity-70" />
              <span class="text-xs text-theme-primary">Show Taps</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer select-none">
              <input
                v-model="store.showTapsEnabled"
                type="checkbox"
                class="sr-only peer"
                @change="store.toggleShowTaps(store.showTapsEnabled)"
              />
              <div
                class="w-8 h-[18px] bg-theme-toggle-track rounded-full peer peer-focus:outline-none peer-checked:bg-accent-emerald after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[#06100d] after:rounded-full after:h-3.5 after:w-3.5 after:shadow-md after:transition-all after:duration-300 after:ease-[cubic-bezier(0.34,1.56,0.64,1)] peer-checked:after:translate-x-[12px] rtl:peer-checked:after:-translate-x-[12px] peer-checked:after:bg-accent-emerald peer-checked:after:bg-accent-emerald peer-active:after:w-[18px]"
              ></div>
            </label>
          </div>

          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <LayoutGrid :size="14" class="opacity-70" />
              <span class="text-xs text-theme-primary">Layout Bounds</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer select-none">
              <input
                v-model="store.layoutBoundsEnabled"
                type="checkbox"
                class="sr-only peer"
                @change="store.toggleLayoutBounds(store.layoutBoundsEnabled)"
              />
              <div
                class="w-8 h-[18px] bg-theme-toggle-track rounded-full peer peer-focus:outline-none peer-checked:bg-accent-emerald after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[#06100d] after:rounded-full after:h-3.5 after:w-3.5 after:shadow-md after:transition-all after:duration-300 after:ease-[cubic-bezier(0.34,1.56,0.64,1)] peer-checked:after:translate-x-[12px] rtl:peer-checked:after:-translate-x-[12px] peer-checked:after:bg-accent-emerald peer-checked:after:bg-accent-emerald peer-active:after:w-[18px]"
              ></div>
            </label>
          </div>
        </div>
      </div>

      <!-- Display -->
      <div ref="displayRef" data-section="display" class="mb-6">
        <h3 class="font-sans text-xs font-semibold tracking-wider mb-4 uppercase text-theme-muted">
          Display
        </h3>
        <div class="flex items-center gap-2">
          <Sun :size="14" class="opacity-70 shrink-0" />
          <input
            v-model.number="store.brightness"
            type="range"
            min="0"
            max="255"
            class="flex-1 accent-theme cursor-pointer min-w-0"
            @change="store.setBrightness(store.brightness)"
          />
          <span class="text-[10px] text-theme-muted w-6 text-right">{{ store.brightness }}</span>
        </div>

        <div
          class="flex items-center gap-2 mt-3"
          :class="{ 'opacity-40 pointer-events-none': !store.connected || !store.currentDensity }"
        >
          <Scaling :size="14" class="opacity-70 shrink-0" />
          <input
            v-model.number="store.currentDensity"
            type="range"
            min="120"
            max="640"
            step="10"
            class="flex-1 accent-theme cursor-pointer min-w-0"
            @change="store.setDensity(store.currentDensity)"
          />
          <span class="text-[10px] text-theme-muted w-8 text-right">{{
            store.currentDensity || '—'
          }}</span>
          <button
            v-if="store.densityOverride !== null"
            class="btn-pressable text-[10px] px-1.5 py-0.5 rounded bg-theme-btn border border-theme-tertiary hover-accent transition-all whitespace-nowrap"
            title="Reset to factory density"
            @click="store.resetDensity()"
          >
            Reset
          </button>
        </div>
      </div>

      <!-- Remote Controls -->
      <div ref="remoteRef" data-section="remote" class="mb-6">
        <h3 class="font-sans text-xs font-semibold tracking-wider mb-4 uppercase text-theme-muted">
          Remote Controls
        </h3>
        <div class="space-y-2">
          <div class="grid grid-cols-3 gap-1.5">
            <button
              class="btn-pressable flex flex-col items-center justify-center gap-0.5 py-2 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
              @click="store.pressHome"
            >
              <Home :size="16" class="opacity-70" />
              <span class="text-[9px] font-medium">Home</span>
            </button>
            <button
              class="btn-pressable flex flex-col items-center justify-center gap-0.5 py-2 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
              @click="store.pressBack"
            >
              <ArrowLeft :size="16" class="opacity-70" />
              <span class="text-[9px] font-medium">Back</span>
            </button>
            <button
              class="btn-pressable flex flex-col items-center justify-center gap-0.5 py-2 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
              @click="store.pressRecent"
            >
              <History :size="16" class="opacity-70" />
              <span class="text-[9px] font-medium">Recent</span>
            </button>
          </div>

          <div class="grid grid-cols-3 gap-1.5">
            <button
              class="btn-pressable flex items-center justify-center py-2 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
              title="Volume Down"
              @click="store.pressVolDown"
            >
              <Volume1 :size="14" class="opacity-70" />
            </button>
            <button
              class="btn-pressable flex items-center justify-center py-2 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
              title="Mute"
              @click="store.pressMute"
            >
              <VolumeX :size="14" class="opacity-70" />
            </button>
            <button
              class="btn-pressable flex items-center justify-center py-2 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
              title="Volume Up"
              @click="store.pressVolUp"
            >
              <Volume2 :size="14" class="opacity-70" />
            </button>
          </div>

          <div class="grid grid-cols-3 gap-1.5">
            <button
              class="btn-pressable flex items-center justify-center py-2 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
              title="Previous"
              @click="store.pressPrev"
            >
              <SkipBack :size="14" class="opacity-70" />
            </button>
            <button
              class="btn-pressable flex items-center justify-center py-2 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
              title="Play/Pause"
              @click="store.pressPlayPause"
            >
              <Play :size="14" class="opacity-70" />
            </button>
            <button
              class="btn-pressable flex items-center justify-center py-2 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
              title="Next"
              @click="store.pressNext"
            >
              <SkipForward :size="14" class="opacity-70" />
            </button>
          </div>

          <button
            class="btn-pressable w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
            @click="store.pressPower"
          >
            <Power :size="16" class="opacity-70" />
            <span class="text-xs font-medium">Power</span>
          </button>

          <div class="flex flex-col gap-1.5">
            <input
              v-model="store.textInput"
              type="text"
              placeholder="Input text..."
              class="w-full bg-theme-input border border-theme-secondary rounded-lg px-2 py-1.5 text-xs text-theme-primary focus:outline-none focus:border-accent-emerald/50 placeholder:text-theme-muted"
              @keydown.enter="store.sendText"
            />
            <button
              class="btn-pressable w-full flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-accent-emerald/10 border border-accent-emerald/25 hover-accent transition-all text-xs font-medium group"
              @click="store.sendText"
            >
              <Send :size="12" class="text-accent-emerald group-hover:text-[var(--text-inverse)]" />
              <span class="text-accent-emerald group-hover:text-[var(--text-inverse)]">Enter</span>
            </button>
          </div>
        </div>
      </div>

      <!-- System -->
      <div ref="systemRef" data-section="system" class="mb-4">
        <h3 class="font-sans text-xs font-semibold tracking-wider mb-4 uppercase text-theme-muted">
          System
        </h3>
        <div class="space-y-1.5">
          <button
            class="btn-pressable w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
            @click="store.takeScreenshot"
          >
            <Image :size="16" class="opacity-70" />
            <span class="text-xs font-medium">Screenshot</span>
          </button>

          <div class="relative reboot-menu-container">
            <button
              class="btn-pressable w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-theme-btn border border-theme-tertiary hover-accent transition-all"
              @click="store.showRebootMenu = !store.showRebootMenu"
            >
              <RotateCcw :size="16" class="opacity-70" />
              <span class="text-xs font-medium">Reboot</span>
              <ChevronDown :size="12" class="opacity-50" />
            </button>
            <div
              v-if="store.showRebootMenu"
              class="absolute left-0 right-0 top-full mt-1 rounded-xl bg-theme-sidebar border border-theme-secondary overflow-hidden z-30"
            >
              <button
                class="btn-pressable w-full px-3 py-2 text-xs text-left hover-accent transition-all flex items-center gap-2 text-theme-primary"
                @click="
                  store.rebootDevice();
                  store.showRebootMenu = false;
                "
              >
                <MonitorUp :size="12" class="opacity-70" />
                Normal Reboot
              </button>
              <button
                class="btn-pressable w-full px-3 py-2 text-xs text-left hover-accent transition-all text-color-warning flex items-center gap-2"
                @click="store.rebootRecovery()"
              >
                <RotateCcw :size="12" class="opacity-70" />
                Recovery
              </button>
              <button
                class="btn-pressable w-full px-3 py-2 text-xs text-left hover-accent transition-all text-[var(--color-tertiary)] flex items-center gap-2"
                @click="store.rebootBootloader()"
              >
                <Power :size="12" class="opacity-70" />
                Bootloader
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>
