<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import { useLogcatStore } from '../stores/logcat';
import { ChevronDown, Activity } from '@lucide/vue';

const store = useLogcatStore();
const container = ref<HTMLDivElement | null>(null);
const userScrolledUp = ref(false);
const newEntriesBadge = ref(false);

function levelColorClass(level: string): string {
  switch (level) {
    case 'V':
      return 'text-gray-400';
    case 'D':
      return 'text-blue-400';
    case 'I':
      return 'text-accent-emerald';
    case 'W':
      return 'text-amber-400';
    case 'E':
      return 'text-red-400';
    case 'F':
      return 'text-pink-400';
    default:
      return 'text-gray-400';
  }
}

function scrollToBottom() {
  if (!container.value) return;
  container.value.scrollTop = container.value.scrollHeight;
  userScrolledUp.value = false;
  newEntriesBadge.value = false;
}

function onScroll() {
  if (!container.value) return;
  const el = container.value;
  const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 80;
  userScrolledUp.value = !nearBottom;
  if (nearBottom) newEntriesBadge.value = false;
}

// Watch: new entries arrive → auto-scroll if enabled and user is at bottom
watch(
  () => store.lastAppendAt,
  () => {
    if (!container.value || store.paused) return;
    if (store.autoScroll && !userScrolledUp.value) {
      nextTick(() => {
        if (container.value) container.value.scrollTop = container.value.scrollHeight;
      });
    } else {
      newEntriesBadge.value = true;
    }
  }
);

// Watch: auto-scroll checkbox toggled ON → scroll immediately
watch(
  () => store.autoScroll,
  (on) => {
    if (on && !store.paused) scrollToBottom();
  }
);
</script>

<template>
  <div
    class="relative flex flex-col min-h-0 border border-theme-tertiary rounded-lg bg-theme-card overflow-hidden"
  >
    <!-- Error / Disconnect banner -->
    <div
      v-if="store.error && (store.status === 'ERROR' || store.status === 'DISCONNECTED')"
      class="absolute top-2 right-1/2 translate-x-1/2 z-20 flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-medium bg-red-500/15 border border-red-500/30 text-red-400 max-w-[80%] text-center"
      @click="store.error = ''"
    >
      ⚠ {{ store.error }}
    </div>

    <!-- New Logs Badge -->
    <div
      v-if="newEntriesBadge"
      class="btn-pressable absolute top-2 right-1/2 translate-x-1/2 z-20 flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-accent-15 border border-accent-30 text-accent-emerald cursor-pointer hover:bg-accent-25"
      @click="scrollToBottom"
    >
      <Activity :size="12" /> New logs
    </div>

    <!-- Terminal Stream (native, no virtual scroll) -->
    <div
      ref="container"
      class="flex-1 overflow-y-auto p-3 font-mono text-xs leading-relaxed"
      @scroll="onScroll"
    >
      <div v-if="store.filteredEntries.length === 0" class="text-theme-muted text-center py-8">
        {{
          store.totalCount === 0
            ? 'No logs yet. Start streaming to see logs.'
            : 'No logs match the current filter.'
        }}
      </div>

      <div v-for="entry in store.filteredEntries" :key="entry.id" class="py-0.5">
        <div class="flex gap-2 items-start">
          <span class="text-[var(--text-muted)] shrink-0 w-[7rem]">{{
            entry.timestamp || '—'
          }}</span>
          <span class="text-[var(--text-muted)] shrink-0 w-[3rem] text-right">{{
            entry.pid || '—'
          }}</span>
          <span class="text-[var(--text-muted)] shrink-0 w-[3rem] text-right">{{
            entry.tid || '—'
          }}</span>
          <span
            class="shrink-0 w-[1.5rem] text-center font-bold"
            :class="levelColorClass(entry.level)"
            >{{ entry.level }}</span
          >
          <span class="font-semibold shrink-0">{{ entry.tag || 'UNKNOWN' }}:</span>
          <span class="break-all">{{ entry.displayMessage }}</span>
        </div>

        <div v-if="entry.hasCont" class="pl-[16.5rem] text-[var(--text-muted)] whitespace-pre-wrap">
          {{ entry.contLines.join('\n') }}
        </div>
      </div>
    </div>

    <!-- Status Bar -->
    <div
      class="flex items-center gap-3 px-3 py-2 border-t border-theme-tertiary bg-theme-btn text-[11px] text-theme-muted shrink-0 z-10"
    >
      <span>
        {{ store.visibleCount.toLocaleString() }} lines (filtered from
        {{ store.totalCount.toLocaleString() }})
      </span>

      <span v-if="store.droppedCount > 0" class="text-amber-400">
        ⚠ {{ store.droppedCount.toLocaleString() }} dropped
      </span>

      <div class="flex-1" />

      <label
        class="btn-pressable flex items-center gap-1.5 cursor-pointer hover:text-theme-secondary"
      >
        <input v-model="store.autoScroll" type="checkbox" class="accent-accent-emerald" />
        Auto-scroll
      </label>

      <button
        class="btn-pressable flex items-center gap-1 hover:text-theme-secondary"
        @click="scrollToBottom"
      >
        <ChevronDown :size="12" /> Bottom
      </button>
    </div>
  </div>
</template>
