<script setup lang="ts">
import { useDeviceStore } from "../stores/device";
import { Link } from "lucide-vue-next";

const store = useDeviceStore();

function handleConnect() {
  if (store.ipAddress.trim()) {
    store.connectWithRetry();
  } else {
    store.autoConnect();
  }
}
</script>

<template>
  <section class="card-glass border border-card-border rounded-2xl p-4">
    <div class="flex items-center gap-2 mb-3">
      <Link :size="14" class="text-accent-emerald" />
      <h2 class="text-xs font-bold uppercase tracking-widest">Connect</h2>
    </div>
    <div class="space-y-3">
      <div class="flex gap-2">
        <div class="flex-1">
          <input v-model="store.ipAddress" type="text" placeholder="IP address (e.g., 192.168.1.5)"
            class="w-full bg-theme-input border border-theme-secondary rounded-lg py-1.5 px-3 text-xs text-theme-primary focus:outline-none focus:border-accent-emerald/50 placeholder:text-theme-muted" />
        </div>
        <div class="w-28">
          <input v-model="store.usbDeviceId" type="text" placeholder="USB ID"
            class="w-full bg-theme-input border border-theme-secondary rounded-lg py-1.5 px-3 text-xs text-theme-primary focus:outline-none focus:border-accent-emerald/50 placeholder:text-theme-muted" />
        </div>
      </div>
      <div class="flex gap-2">
        <button @click="handleConnect" :disabled="store.connecting"
          class="flex-1 bg-accent-emerald hover:bg-accent-emerald-hover text-white py-2 rounded-lg text-xs font-semibold transition-colors shadow-lg shadow-emerald-500/20 disabled:opacity-50 disabled:cursor-not-allowed">
          {{ store.connecting ? "Connecting..." : "Auto Connect" }}
        </button>
        <button v-if="store.connected" @click="store.disconnect"
          class="bg-theme-btn border border-theme-secondary text-theme-secondary py-2 px-4 rounded-lg text-xs hover:bg-theme-hover transition-colors">
          Disconnect
        </button>
      </div>
    </div>
  </section>
</template>