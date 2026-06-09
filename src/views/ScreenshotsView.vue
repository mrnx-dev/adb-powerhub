<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { Camera, Loader2, RefreshCw } from '@lucide/vue';
import { useScreenshotsStore, type SortMode, type FilterMode } from '../stores/screenshots';
import { useDeviceStore } from '../stores/device';
import { useSettingsStore } from '../stores/settings';
import { useNavigationStore } from '../stores/navigation';
import { useToastStore } from '../stores/toast';
import ScreenshotGrid from '../components/ScreenshotGrid.vue';
import ScreenshotLightbox from '../components/ScreenshotLightbox.vue';
import ScreenshotTruncationBanner from '../components/ScreenshotTruncationBanner.vue';

const store = useScreenshotsStore();
const deviceStore = useDeviceStore();
const settingsStore = useSettingsStore();
const navStore = useNavigationStore();
const toast = useToastStore();

const capturing = ref(false);

const sortOptions: { value: SortMode; label: string }[] = [
  { value: 'newest', label: 'Newest' },
  { value: 'oldest', label: 'Oldest' },
  { value: 'largest', label: 'Largest' },
];

const filterOptions: { value: FilterMode; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'today', label: 'Today' },
  { value: 'week', label: 'This Week' },
];

const dirDisplay = computed(() => {
  const dir = settingsStore.screenshotSaveDir || '~/Pictures/adb-powerhub';
  // Truncate long paths
  if (dir.length > 40) {
    return '…' + dir.slice(-37);
  }
  return dir;
});

async function handleCapture() {
  if (!deviceStore.connected || capturing.value) return;
  capturing.value = true;
  try {
    await deviceStore.takeScreenshot();
  } catch (e) {
    toast.show(`Screenshot failed: ${e}`, 'error');
  } finally {
    capturing.value = false;
  }
}

// Watch for settings directory changes
watch(
  () => settingsStore.screenshotSaveDir,
  () => {
    if (navStore.currentPage === 'screenshots') {
      store.refresh();
    }
  }
);

onMounted(() => {
  store.refresh();
});
</script>

<template>
  <div tabindex="-1" class="flex flex-col h-full overflow-hidden outline-none">
    <!-- Toolbar -->
    <div
      class="flex items-center gap-3 px-4 py-3 border-b border-theme-tertiary shrink-0 flex-wrap"
    >
      <!-- Take Screenshot -->
      <button
        class="btn-pressable flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium"
        :class="
          deviceStore.connected
            ? 'bg-accent-emerald text-white hover:bg-accent-emerald/90'
            : 'bg-theme-btn text-theme-muted cursor-not-allowed'
        "
        :disabled="!deviceStore.connected || capturing"
        :title="!deviceStore.connected ? 'Connect a device to take screenshots' : undefined"
        @click="handleCapture"
      >
        <Loader2 v-if="capturing" :size="16" class="animate-spin" />
        <Camera v-else :size="16" />
        <span>{{ capturing ? 'Capturing…' : 'Take Screenshot' }}</span>
      </button>

      <div class="flex-1" />

      <!-- Refresh button -->
      <button
        class="btn-pressable p-2 rounded-lg text-theme-muted hover:text-theme-primary hover:bg-theme-btn transition-colors"
        :disabled="store.loading"
        aria-label="Refresh screenshots"
        :title="store.loading ? 'Refreshing…' : 'Refresh screenshots'"
        @click="store.refresh(undefined, true)"
      >
        <Loader2 v-if="store.loading" :size="16" class="animate-spin" />
        <RefreshCw v-else :size="16" />
      </button>

      <!-- Sort dropdown -->
      <select
        :value="store.sortMode"
        class="bg-theme-btn border border-theme-tertiary rounded-lg px-2 py-1.5 text-xs text-theme-secondary focus:outline-none focus:border-accent-emerald/50"
        @change="store.setSort(($event.target as HTMLSelectElement).value as SortMode)"
      >
        <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>

      <!-- Filter pills -->
      <div class="flex gap-1">
        <button
          v-for="f in filterOptions"
          :key="f.value"
          class="btn-pressable px-2.5 py-1 rounded-md text-xs font-medium transition-colors duration-150"
          :class="
            store.filterMode === f.value
              ? 'bg-accent-emerald text-white'
              : 'bg-theme-btn text-theme-muted border border-theme-tertiary hover:border-accent-emerald/40'
          "
          @click="store.setFilter(f.value)"
        >
          {{ f.label }}
        </button>
      </div>

      <!-- Breadcrumb -->
      <span class="text-xs text-theme-muted whitespace-nowrap">📁 {{ dirDisplay }}</span>
      <button
        class="btn-pressable text-xs text-accent-emerald hover:underline whitespace-nowrap"
        @click="navStore.navigateTo('settings')"
      >
        Change
      </button>
    </div>

    <!-- Content area -->
    <div class="flex-1 overflow-y-auto p-4">
      <!-- Truncation banner -->
      <ScreenshotTruncationBanner
        v-if="store.isTruncated && !store.truncationBannerDismissed"
        :total-count="store.totalCount"
        :shown-count="store.files.length"
        @dismiss="store.truncationBannerDismissed = true"
      />
      <div class="mb-3" />
      <!-- Loading: skeleton grid -->
      <div v-if="store.loading" class="grid gap-3 grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
        <div
          v-for="i in 12"
          :key="i"
          class="aspect-[9/19.5] rounded-lg bg-theme-card animate-pulse"
        />
      </div>

      <!-- Error banner -->
      <div v-else-if="store.error" class="flex flex-col items-center justify-center gap-3 py-16">
        <p class="text-theme-muted text-sm">{{ store.error }}</p>
        <button
          class="btn-pressable px-4 py-2 rounded-lg bg-theme-btn border border-theme-tertiary text-sm hover-subtle"
          @click="store.refresh(undefined, true)"
        >
          Retry
        </button>
      </div>

      <!-- Full empty state -->
      <div v-else-if="store.isEmpty" class="flex flex-col items-center justify-center gap-4 py-20">
        <div class="w-16 h-16 rounded-full bg-theme-card flex items-center justify-center">
          <Camera :size="32" class="text-theme-muted" />
        </div>
        <p class="text-theme-primary font-medium">No screenshots yet</p>
        <p class="text-theme-muted text-sm">Take a screenshot to see it here</p>
        <button
          class="btn-pressable flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium"
          :class="
            deviceStore.connected
              ? 'bg-accent-emerald text-white'
              : 'bg-theme-btn text-theme-muted cursor-not-allowed'
          "
          :disabled="!deviceStore.connected"
          @click="handleCapture"
        >
          <Camera :size="16" />
          Take Screenshot
        </button>
      </div>

      <!-- Filtered empty state -->
      <div
        v-else-if="store.isEmptyFiltered"
        class="flex flex-col items-center justify-center gap-3 py-16"
      >
        <p class="text-theme-muted text-sm">No screenshots match this filter</p>
        <button
          class="btn-pressable px-4 py-2 rounded-lg bg-theme-btn border border-theme-tertiary text-sm hover-subtle"
          @click="store.setFilter('all')"
        >
          Clear filter
        </button>
      </div>

      <!-- Grid + Lightbox -->
      <template v-else>
        <ScreenshotGrid @select="store.openLightbox" />
        <ScreenshotLightbox />
      </template>
    </div>
  </div>
</template>
