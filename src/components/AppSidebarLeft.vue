<script setup lang="ts">
import { ref } from "vue";
import { useNavigationStore } from "../stores/navigation";
import { LayoutDashboard, Settings, PanelLeftOpen, PanelLeftClose } from "lucide-vue-next";

const navStore = useNavigationStore();
const expanded = ref(false);
</script>

<template>
  <aside
    class="bg-sidebar-dark sidebar-blur border-r border-white/5 flex flex-col shrink-0 transition-all duration-200 overflow-hidden"
    :class="expanded ? 'w-56' : 'w-12'">

    <div class="flex items-center shrink-0" :class="expanded ? 'justify-between px-4 py-3' : 'justify-center py-3'">
      <button @click="expanded = !expanded"
        class="p-1.5 rounded-lg hover:bg-white/10 transition-all"
        :title="expanded ? 'Collapse sidebar' : 'Expand sidebar'">
        <PanelLeftClose v-if="expanded" :size="18" class="opacity-70" />
        <PanelLeftOpen v-else :size="18" class="opacity-70" />
      </button>
    </div>

    <div class="flex flex-col" :class="expanded ? 'px-3 gap-1' : 'px-2 gap-1'">
      <h3 v-if="expanded" class="text-[10px] font-bold text-gray-500 tracking-widest mb-1 uppercase px-2">Navigation</h3>

      <button @click="navStore.navigateTo('dashboard')"
        class="rounded-xl border transition-all"
        :class="[
          navStore.currentPage === 'dashboard'
            ? 'bg-accent-emerald/10 border-accent-emerald/50 text-accent-emerald'
            : 'bg-white/5 border-white/5 text-gray-300 hover:bg-white/10 hover:border-accent-emerald/50',
          expanded ? 'flex items-center gap-3 px-2 py-2.5' : 'flex items-center justify-center py-2.5'
        ]"
        :title="!expanded ? 'Dashboard' : undefined">
        <LayoutDashboard :size="18" class="shrink-0" />
        <span v-if="expanded" class="text-sm font-medium whitespace-nowrap">Dashboard</span>
      </button>

      <button @click="navStore.navigateTo('settings')"
        class="rounded-xl border transition-all"
        :class="[
          navStore.currentPage === 'settings'
            ? 'bg-accent-emerald/10 border-accent-emerald/50 text-accent-emerald'
            : 'bg-white/5 border-white/5 text-gray-300 hover:bg-white/10 hover:border-accent-emerald/50',
          expanded ? 'flex items-center gap-3 px-2 py-2.5' : 'flex items-center justify-center py-2.5'
        ]"
        :title="!expanded ? 'Settings' : undefined">
        <Settings :size="18" class="shrink-0" />
        <span v-if="expanded" class="text-sm font-medium whitespace-nowrap">Settings</span>
      </button>
    </div>
  </aside>
</template>