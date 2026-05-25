<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import { useDeviceStore } from "../stores/device";
import {
  Wifi, ArrowUpDown, Plane, Bluetooth,
  Pointer, LayoutGrid, Sun,
  Home, ArrowLeft, History,
  Volume1, Volume2, VolumeX,
  Power, SkipBack, Play, SkipForward,
  Send, Image,
  PanelRightClose, PanelRightOpen,
  ChevronDown, RotateCcw, MonitorUp,
  Smartphone,
} from "lucide-vue-next";

const store = useDeviceStore();
const collapsed = ref(false);

const connectivityRef = ref<HTMLElement | null>(null);
const devToolsRef = ref<HTMLElement | null>(null);
const displayRef = ref<HTMLElement | null>(null);
const remoteRef = ref<HTMLElement | null>(null);
const systemRef = ref<HTMLElement | null>(null);

function expandAndScroll(refVal: HTMLElement | null) {
  collapsed.value = false;
  nextTick(() => {
    refVal?.scrollIntoView({ behavior: "smooth", block: "start" });
  });
}

function handleClickOutside(e: MouseEvent) {
  if (store.showRebootMenu) {
    const target = e.target as HTMLElement;
    if (!target.closest(".reboot-menu-container")) {
      store.showRebootMenu = false;
    }
  }
}

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
});
</script>

<template>
  <aside
    class="bg-sidebar-dark sidebar-blur border-l border-white/5 flex flex-col shrink-0 overflow-hidden transition-all duration-200 z-20"
    :class="collapsed ? 'w-12' : 'w-48'">

    <div class="flex items-center shrink-0" :class="collapsed ? 'justify-center p-3' : 'justify-end p-2'">
      <button @click="collapsed = !collapsed"
        class="p-1.5 rounded-lg hover:bg-white/10 transition-all"
        :title="collapsed ? 'Expand sidebar' : 'Collapse sidebar'">
        <PanelRightOpen v-if="collapsed" :size="16" class="opacity-70" />
        <PanelRightClose v-else :size="16" class="opacity-70" />
      </button>
    </div>

    <!-- Collapsed: section icons -->
    <div v-if="collapsed" class="flex flex-col items-center gap-1 px-2 mt-1">
      <button @click="expandAndScroll(connectivityRef)" class="p-2.5 rounded-xl hover:bg-white/10 transition-all" title="Connectivity">
        <Wifi :size="16" class="opacity-70" />
      </button>
      <button @click="expandAndScroll(devToolsRef)" class="p-2.5 rounded-xl hover:bg-white/10 transition-all" title="Dev Tools">
        <Pointer :size="16" class="opacity-70" />
      </button>
      <button @click="expandAndScroll(displayRef)" class="p-2.5 rounded-xl hover:bg-white/10 transition-all" title="Display">
        <Sun :size="16" class="opacity-70" />
      </button>
      <button @click="expandAndScroll(remoteRef)" class="p-2.5 rounded-xl hover:bg-white/10 transition-all" title="Remote Controls">
        <Smartphone :size="16" class="opacity-70" />
      </button>
      <button @click="expandAndScroll(systemRef)" class="p-2.5 rounded-xl hover:bg-white/10 transition-all" title="System">
        <Image :size="16" class="opacity-70" />
      </button>
    </div>

    <!-- Expanded: full content -->
    <div v-if="!collapsed" class="flex flex-col p-3 overflow-y-auto">

      <!-- Connectivity -->
      <div ref="connectivityRef" class="mb-6">
        <h3 class="text-[10px] font-bold text-gray-500 tracking-widest mb-4 uppercase">Connectivity</h3>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Wifi :size="14" class="opacity-70" />
              <span class="text-xs">Wi-Fi</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" class="sr-only peer" v-model="store.wifiEnabled"
                @change="store.toggleWifi(store.wifiEnabled)" />
              <div class="w-8 h-[18px] bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-accent-emerald"></div>
            </label>
          </div>

          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <ArrowUpDown :size="14" class="opacity-70" />
              <span class="text-xs">Data</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" class="sr-only peer" v-model="store.dataEnabled"
                @change="store.toggleData(store.dataEnabled)" />
              <div class="w-8 h-[18px] bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-accent-emerald"></div>
            </label>
          </div>

          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Plane :size="14" class="opacity-70" />
              <span class="text-xs">Airplane</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" class="sr-only peer" v-model="store.airplaneEnabled"
                @change="store.toggleAirplane(store.airplaneEnabled)" />
              <div class="w-8 h-[18px] bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-accent-emerald"></div>
            </label>
          </div>

          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Bluetooth :size="14" class="opacity-70" />
              <span class="text-xs">Bluetooth</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" class="sr-only peer" v-model="store.bluetoothEnabled"
                @change="store.toggleBluetooth(store.bluetoothEnabled)" />
              <div class="w-8 h-[18px] bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-accent-emerald"></div>
            </label>
          </div>
        </div>
      </div>

      <!-- Dev Tools -->
      <div ref="devToolsRef" class="mb-6">
        <h3 class="text-[10px] font-bold text-gray-500 tracking-widest mb-4 uppercase">Dev Tools</h3>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Pointer :size="14" class="opacity-70" />
              <span class="text-xs">Show Taps</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" class="sr-only peer" v-model="store.showTapsEnabled"
                @change="store.toggleShowTaps(store.showTapsEnabled)" />
              <div class="w-8 h-[18px] bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-accent-emerald"></div>
            </label>
          </div>

          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <LayoutGrid :size="14" class="opacity-70" />
              <span class="text-xs">Layout Bounds</span>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" class="sr-only peer" v-model="store.layoutBoundsEnabled"
                @change="store.toggleLayoutBounds(store.layoutBoundsEnabled)" />
              <div class="w-8 h-[18px] bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-accent-emerald"></div>
            </label>
          </div>
        </div>
      </div>

      <!-- Display -->
      <div ref="displayRef" class="mb-6">
        <h3 class="text-[10px] font-bold text-gray-500 tracking-widest mb-4 uppercase">Display</h3>
        <div class="flex items-center gap-2">
          <Sun :size="14" class="opacity-70 shrink-0" />
          <input type="range" min="0" max="255" v-model.number="store.brightness"
            @change="store.setBrightness(store.brightness)"
            class="flex-1 h-1.5 bg-gray-700 rounded-full appearance-none cursor-pointer accent-emerald-500" />
          <span class="text-[10px] text-gray-500 w-6 text-right">{{ store.brightness }}</span>
        </div>
      </div>

      <!-- Remote Controls -->
      <div ref="remoteRef" class="mb-6">
        <h3 class="text-[10px] font-bold text-gray-500 tracking-widest mb-4 uppercase">Remote Controls</h3>
        <div class="space-y-2">
          <div class="grid grid-cols-3 gap-1.5">
            <button @click="store.pressHome"
              class="flex flex-col items-center justify-center gap-0.5 py-2 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all">
              <Home :size="16" class="opacity-70" />
              <span class="text-[9px] font-medium">Home</span>
            </button>
            <button @click="store.pressBack"
              class="flex flex-col items-center justify-center gap-0.5 py-2 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all">
              <ArrowLeft :size="16" class="opacity-70" />
              <span class="text-[9px] font-medium">Back</span>
            </button>
            <button @click="store.pressRecent"
              class="flex flex-col items-center justify-center gap-0.5 py-2 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all">
              <History :size="16" class="opacity-70" />
              <span class="text-[9px] font-medium">Recent</span>
            </button>
          </div>

          <div class="grid grid-cols-3 gap-1.5">
            <button @click="store.pressVolDown"
              class="flex items-center justify-center py-2 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all"
              title="Volume Down">
              <Volume1 :size="14" class="opacity-70" />
            </button>
            <button @click="store.pressMute"
              class="flex items-center justify-center py-2 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all"
              title="Mute">
              <VolumeX :size="14" class="opacity-70" />
            </button>
            <button @click="store.pressVolUp"
              class="flex items-center justify-center py-2 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all"
              title="Volume Up">
              <Volume2 :size="14" class="opacity-70" />
            </button>
          </div>

          <div class="grid grid-cols-3 gap-1.5">
            <button @click="store.pressPrev"
              class="flex items-center justify-center py-2 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all"
              title="Previous">
              <SkipBack :size="14" class="opacity-70" />
            </button>
            <button @click="store.pressPlayPause"
              class="flex items-center justify-center py-2 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all"
              title="Play/Pause">
              <Play :size="14" class="opacity-70" />
            </button>
            <button @click="store.pressNext"
              class="flex items-center justify-center py-2 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all"
              title="Next">
              <SkipForward :size="14" class="opacity-70" />
            </button>
          </div>

          <button @click="store.pressPower"
            class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all">
            <Power :size="16" class="opacity-70" />
            <span class="text-xs font-medium">Power</span>
          </button>

          <div class="flex flex-col gap-1.5">
            <input v-model="store.textInput" @keydown.enter="store.sendText"
              type="text" placeholder="Input text..."
              class="w-full bg-black/40 border border-white/10 rounded-lg px-2 py-1.5 text-xs focus:outline-none focus:border-accent-emerald/50" />
            <button @click="store.sendText"
              class="w-full flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-accent-emerald/20 border border-accent-emerald/30 hover:bg-accent-emerald/30 transition-all text-xs font-medium">
              <Send :size="12" class="text-accent-emerald" />
              <span class="text-accent-emerald">Enter</span>
            </button>
          </div>
        </div>
      </div>

      <!-- System -->
      <div ref="systemRef" class="mb-4">
        <h3 class="text-[10px] font-bold text-gray-500 tracking-widest mb-4 uppercase">System</h3>
        <div class="space-y-1.5">
          <button @click="store.takeScreenshot"
            class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all">
            <Image :size="16" class="opacity-70" />
            <span class="text-xs font-medium">Screenshot</span>
          </button>

          <div class="relative reboot-menu-container">
            <button @click="store.showRebootMenu = !store.showRebootMenu"
              class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all">
              <RotateCcw :size="16" class="opacity-70" />
              <span class="text-xs font-medium">Reboot</span>
              <ChevronDown :size="12" class="opacity-50" />
            </button>
            <div v-if="store.showRebootMenu"
              class="absolute left-0 right-0 top-full mt-1 rounded-xl bg-sidebar-dark border border-white/10 overflow-hidden z-30">
              <button @click="store.rebootDevice(); store.showRebootMenu = false"
                class="w-full px-3 py-2 text-xs text-left hover:bg-white/10 transition-all flex items-center gap-2">
                <MonitorUp :size="12" class="opacity-70" />
                Normal Reboot
              </button>
              <button @click="store.rebootRecovery()"
                class="w-full px-3 py-2 text-xs text-left hover:bg-white/10 transition-all text-yellow-400/80 flex items-center gap-2">
                <RotateCcw :size="12" class="opacity-70" />
                Recovery
              </button>
              <button @click="store.rebootBootloader()"
                class="w-full px-3 py-2 text-xs text-left hover:bg-white/10 transition-all text-orange-400/80 flex items-center gap-2">
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