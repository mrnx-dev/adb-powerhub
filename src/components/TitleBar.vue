<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-vue-next";

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
  <header class="flex items-center justify-between px-4 py-2 bg-sidebar-dark border-b border-white/5 select-none z-50"
    data-tauri-drag-region>
    <div class="flex items-center gap-2" data-tauri-drag-region>
      <span class="text-sm font-medium opacity-80">ADB PowerHub</span>
    </div>
    <div class="flex items-center gap-4 text-gray-400" data-tauri-drag-region="false">
      <button @click="minimize" class="cursor-pointer hover:text-white transition-colors p-1" aria-label="Minimize">
        <Minus :size="16" />
      </button>
      <button @click="toggleMaximize" class="cursor-pointer hover:text-white transition-colors p-1" aria-label="Maximize">
        <Square :size="14" />
      </button>
      <button @click="close" class="cursor-pointer hover:text-red-500 transition-colors p-1" aria-label="Close">
        <X :size="16" />
      </button>
    </div>
  </header>
</template>