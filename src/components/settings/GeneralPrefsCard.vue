<script setup lang="ts">
import { useSettingsStore } from '../../stores/settings';
import { Settings } from '@lucide/vue';

const store = useSettingsStore();

const pollingOptions = [1, 2, 3, 5, 10, 15, 30];
</script>

<template>
  <section class="card-glass p-4">
    <div class="flex items-center gap-2 mb-4">
      <Settings :size="16" class="text-accent-emerald" />
      <h2 class="font-sans text-xs font-semibold tracking-wider uppercase">General</h2>
    </div>

    <div class="space-y-5">
      <!-- Stay on Top -->
      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Stay on Top</span>
          <p class="text-[10px] text-theme-muted">Keep window above other windows</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer select-none">
          <input
            v-model="store.stayOnTop"
            type="checkbox"
            class="sr-only peer"
            @change="store.setStayOnTop(store.stayOnTop)"
          />
          <div
            class="w-9 h-5 bg-theme-toggle-track rounded-full peer peer-focus:outline-none after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[#06100d] after:rounded-full after:h-4 after:w-4 after:shadow-md after:transition-all after:duration-300 after:ease-[cubic-bezier(0.34,1.56,0.64,1)] peer-checked:after:translate-x-[14px] rtl:peer-checked:after:-translate-x-[14px] peer-checked:after:bg-accent-emerald peer-active:after:w-[22px]"
          ></div>
        </label>
      </div>

      <!-- Auto-detect Binaries -->
      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Auto-detect Binaries</span>
          <p class="text-[10px] text-theme-muted">Scan PATH for adb and scrcpy on startup</p>
        </div>
        <label class="relative inline-flex items-center cursor-pointer select-none">
          <input
            v-model="store.autoDetectBinaries"
            type="checkbox"
            class="sr-only peer"
            @change="store.setAutoDetectBinaries(store.autoDetectBinaries)"
          />
          <div
            class="w-9 h-5 bg-theme-toggle-track rounded-full peer peer-focus:outline-none peer-checked:bg-accent-emerald after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[#06100d] after:rounded-full after:h-4 after:w-4 after:shadow-md after:transition-all after:duration-300 after:ease-[cubic-bezier(0.34,1.56,0.64,1)] peer-checked:after:translate-x-[14px] rtl:peer-checked:after:-translate-x-[14px] peer-checked:after:bg-accent-emerald peer-active:after:w-[22px]"
          ></div>
        </label>
      </div>

      <!-- Polling Interval -->
      <div>
        <label class="text-sm block mb-2">Device Stats Polling Interval</label>
        <select
          v-model.number="store.pollingInterval"
          class="w-full bg-theme-input border border-theme-secondary rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-emerald/50 cursor-pointer"
          @change="store.setPollingInterval(store.pollingInterval)"
        >
          <option v-for="opt in pollingOptions" :key="opt" :value="opt" class="bg-theme-card">
            {{ opt }} second{{ opt > 1 ? 's' : '' }}
          </option>
        </select>
      </div>
    </div>
  </section>
</template>
