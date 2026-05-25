<script setup lang="ts">
import { useDeviceStore } from "../stores/device";
import { Smartphone, Battery, Cpu, Unlink } from "lucide-vue-next";

const store = useDeviceStore();
</script>

<template>
  <section class="card-glass border border-card-border rounded-2xl p-4">
    <!-- Header with Disconnect -->
    <div v-if="store.connected" class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <Smartphone :size="14" class="text-accent-emerald" />
        <h2 class="text-xs font-bold uppercase tracking-widest">Device Stats</h2>
      </div>
      <button @click="store.disconnect"
        class="p-1.5 rounded-lg bg-theme-btn border border-theme-tertiary hover:bg-red-500/10 hover:border-red-500/30 text-theme-secondary hover:text-red-400 transition-all"
        title="Disconnect">
        <Unlink :size="14" />
      </button>
    </div>

    <!-- Skeleton loading state -->
    <template v-if="store.connected && store.isLoadingStats">
      <div class="flex items-center gap-6 flex-wrap">
        <div class="flex items-center gap-2 w-48">
          <div class="w-3 h-3 rounded-full bg-theme-btn animate-pulse"></div>
          <div class="flex-1 h-3 bg-theme-btn rounded animate-pulse"></div>
          <div class="w-12 h-3 bg-theme-hover rounded animate-pulse"></div>
        </div>
        <div class="h-4 w-px bg-theme-secondary"></div>
        <div class="flex items-center gap-2 w-24">
          <div class="w-3 h-3 rounded-full bg-theme-btn animate-pulse"></div>
          <div class="flex-1 h-4 bg-theme-btn rounded animate-pulse"></div>
        </div>
        <div class="h-4 w-px bg-theme-secondary"></div>
        <div class="flex items-center gap-2 min-w-[140px] w-40">
          <div class="w-3 h-3 rounded-full bg-theme-btn animate-pulse"></div>
          <div class="w-6 h-3 bg-theme-hover rounded animate-pulse"></div>
          <div class="flex-1 h-[6px] bg-theme-hover rounded-full animate-pulse"></div>
          <div class="w-8 h-3 bg-theme-hover rounded animate-pulse"></div>
        </div>
      </div>
    </template>

    <!-- Actual data -->
    <template v-else-if="store.connected">
      <div class="flex items-center gap-6 flex-wrap">
        <div class="flex items-center gap-2">
          <Smartphone :size="14" class="text-accent-emerald shrink-0" />
          <span class="text-xs font-semibold text-theme-primary">{{ store.model }}</span>
          <span class="text-[10px] text-theme-muted">Android {{ store.androidVersion }}</span>
          <span class="text-[10px] text-theme-muted">SDK {{ store.sdkVersion }}</span>
        </div>

        <div class="h-4 w-px bg-theme-secondary"></div>

        <div class="flex items-center gap-2">
          <Battery :size="14" class="shrink-0" :class="store.batteryLevel > 20 ? 'text-green-400' : 'text-red-400'" />
          <span class="text-sm font-bold leading-none">{{ store.batteryLevel }}%</span>
          <span class="text-[10px]" :class="store.batteryColor">{{ store.batteryStatus }}</span>
        </div>

        <div class="h-4 w-px bg-theme-secondary"></div>

        <div class="flex items-center gap-2 min-w-[140px]">
          <Cpu :size="14" class="text-accent-emerald shrink-0" />
          <span class="text-[10px] text-theme-secondary w-7">CPU</span>
          <div class="flex-1 bg-theme-btn rounded-full h-[6px]">
            <div class="bg-accent-emerald h-[6px] rounded-full transition-all duration-500" :style="{ width: store.cpuUsage + '%' }"></div>
          </div>
          <span class="text-[10px] text-theme-secondary w-8 text-right">{{ store.cpuUsage.toFixed(0) }}%</span>
        </div>
      </div>
    </template>

    <!-- No device -->
    <template v-else>
      <div class="flex items-center gap-3 text-theme-muted">
        <Smartphone :size="16" class="opacity-30" />
        <span class="text-xs">No device connected</span>
      </div>
    </template>
  </section>
</template>
