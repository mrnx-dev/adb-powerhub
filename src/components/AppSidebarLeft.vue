<script setup lang="ts">
import { ref } from 'vue';
import { useNavigationStore } from '../stores/navigation';
import { LayoutDashboard, Settings, PanelLeftOpen, PanelLeftClose } from '@lucide/vue';

const navStore = useNavigationStore();
const expanded = ref(false);
</script>

<template>
  <aside
    class="bg-sidebar-dark sidebar-blur border-r border-theme-tertiary flex flex-col shrink-0 transition-all duration-200 overflow-hidden"
    :class="expanded ? 'w-56' : 'w-12'"
  >
    <div
      class="flex items-center shrink-0"
      :class="expanded ? 'justify-between px-4 py-3' : 'justify-center py-3'"
    >
      <button
        class="p-1.5 rounded-lg hover:bg-theme-hover transition-all"
        :title="expanded ? 'Collapse sidebar' : 'Expand sidebar'"
        @click="expanded = !expanded"
      >
        <PanelLeftClose v-if="expanded" :size="18" class="opacity-70" />
        <PanelLeftOpen v-else :size="18" class="opacity-70" />
      </button>
    </div>

    <div class="flex flex-col" :class="expanded ? 'px-3 gap-1' : 'px-2 gap-1'">
      <h3
        v-if="expanded"
        class="font-sans text-xs font-semibold tracking-wider mb-1 uppercase px-2 text-theme-muted"
      >
        Navigation
      </h3>

      <button
        class="rounded-lg border transition-all"
        :class="[
          navStore.currentPage === 'dashboard'
            ? 'bg-accent-emerald/10 border-accent-emerald/25 text-accent-emerald'
            : 'bg-theme-btn border-theme-tertiary text-theme-secondary hover-accent',
          expanded
            ? 'flex items-center gap-3 px-2 py-2.5'
            : 'flex items-center justify-center py-2.5',
        ]"
        :title="!expanded ? 'Dashboard' : undefined"
        @click="navStore.navigateTo('dashboard')"
      >
        <LayoutDashboard :size="18" class="shrink-0" />
        <span v-if="expanded" class="text-sm font-medium whitespace-nowrap">Dashboard</span>
      </button>

      <button
        class="rounded-lg border transition-all"
        :class="[
          navStore.currentPage === 'settings'
            ? 'bg-accent-emerald/10 border-accent-emerald/25 text-accent-emerald'
            : 'bg-theme-btn border-theme-tertiary text-theme-secondary hover-accent',
          expanded
            ? 'flex items-center gap-3 px-2 py-2.5'
            : 'flex items-center justify-center py-2.5',
        ]"
        :title="!expanded ? 'Settings' : undefined"
        @click="navStore.navigateTo('settings')"
      >
        <Settings :size="18" class="shrink-0" />
        <span v-if="expanded" class="text-sm font-medium whitespace-nowrap">Settings</span>
      </button>
    </div>
  </aside>
</template>
