<script setup lang="ts">
import { useDeviceStore } from "../stores/device";
import { Smartphone, Battery, Cpu } from "lucide-vue-next";

const store = useDeviceStore();
</script>

<template>
  <section class="card-glass border border-card-border rounded-2xl p-4">
    <template v-if="store.connected">
      <div class="flex items-center gap-6 flex-wrap">
        <div class="flex items-center gap-2">
          <Smartphone :size="14" class="text-accent-emerald shrink-0" />
          <span class="text-xs font-semibold text-gray-200">{{ store.model }}</span>
          <span class="text-[10px] text-gray-500">Android {{ store.androidVersion }}</span>
          <span class="text-[10px] text-gray-600">SDK {{ store.sdkVersion }}</span>
        </div>

        <div class="h-4 w-px bg-white/10"></div>

        <div class="flex items-center gap-2">
          <Battery :size="14" class="shrink-0" :class="store.batteryLevel > 20 ? 'text-green-400' : 'text-red-400'" />
          <span class="text-sm font-bold leading-none">{{ store.batteryLevel }}%</span>
          <span class="text-[10px]" :class="store.batteryColor">{{ store.batteryStatus }}</span>
        </div>

        <div class="h-4 w-px bg-white/10"></div>

        <div class="flex items-center gap-2 min-w-[140px]">
          <Cpu :size="14" class="text-accent-emerald shrink-0" />
          <span class="text-[10px] text-gray-400 w-7">CPU</span>
          <div class="flex-1 bg-gray-700 rounded-full h-[6px]">
            <div class="bg-accent-emerald h-[6px] rounded-full transition-all duration-500" :style="{ width: store.cpuUsage + '%' }"></div>
          </div>
          <span class="text-[10px] text-gray-400 w-8 text-right">{{ store.cpuUsage.toFixed(0) }}%</span>
        </div>
      </div>
    </template>

    <template v-else>
      <div class="flex items-center gap-3 text-gray-500">
        <Smartphone :size="16" class="opacity-30" />
        <span class="text-xs">No device connected</span>
      </div>
    </template>
  </section>
</template>