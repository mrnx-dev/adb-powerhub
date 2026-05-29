<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X } from '@lucide/vue';

const appWindow = getCurrentWindow();

function minimize() {
  appWindow.minimize();
}
function toggleMaximize() {
  appWindow.toggleMaximize();
}
function close() {
  appWindow.close();
}
</script>

<template>
  <header
    class="flex items-center justify-between px-4 py-2 bg-sidebar-dark border-b border-theme-tertiary select-none z-50"
    data-tauri-drag-region
  >
    <div class="flex items-center gap-2" data-tauri-drag-region>
      <span class="text-sm font-sans font-semibold tracking-wide opacity-80">ADB PowerHub</span>
    </div>
    <div class="flex items-center gap-4 text-theme-secondary" data-tauri-drag-region="false">
      <button
        class="cursor-pointer hover:text-theme-primary transition-colors p-1"
        aria-label="Minimize"
        @click="minimize"
      >
        <Minus :size="16" />
      </button>
      <button
        class="cursor-pointer hover:text-theme-primary transition-colors p-1"
        aria-label="Maximize"
        @click="toggleMaximize"
      >
        <Square :size="14" />
      </button>
      <button
        class="cursor-pointer hover:text-color-error transition-colors p-1"
        aria-label="Close"
        @click="close"
      >
        <X :size="16" />
      </button>
    </div>
  </header>
</template>
