<script setup lang="ts">
import { ref, nextTick, watch, onMounted, onBeforeUnmount, onUnmounted, computed } from 'vue';
import { FolderOpen, Monitor, Copy, Trash2, ImageOff, Loader2, RotateCw } from '@lucide/vue';
import { useScreenshotsStore, type ScreenshotFile } from '../stores/screenshots';

const store = useScreenshotsStore();

const emit = defineEmits<{
  select: [index: number, sourceRect?: DOMRect];
}>();

// ── Lazy loading via IntersectionObserver ──
const visiblePaths = ref<Set<string>>(new Set());
const cardObserver = ref<IntersectionObserver | null>(null);

function initObserver() {
  cardObserver.value = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          const path = (entry.target as HTMLElement).dataset.path;
          if (path) {
            visiblePaths.value = new Set([...visiblePaths.value, path]);
          }
          cardObserver.value?.unobserve(entry.target);
        }
      }
    },
    { rootMargin: '200%' }
  );
}

function observeCard(el: HTMLElement | null, path: string) {
  if (el && cardObserver.value) {
    el.dataset.path = path;
    cardObserver.value.observe(el);
  }
}

// Re-initialize observer when displayed files change
onMounted(() => initObserver());
watch(
  () => store.displayedFiles,
  () => {
    nextTick(() => {
      visiblePaths.value = new Set();
      if (cardObserver.value) cardObserver.value.disconnect();
      initObserver();
      // Reset all thumbnail states
      thumbnailStates.value = new Map();
    });
  }
);
onBeforeUnmount(() => cardObserver.value?.disconnect());

// ── Per-card thumbnail load state ──
const thumbnailStates = ref<Map<string, 'loading' | 'loaded' | 'broken'>>(new Map());

// Watch for newly visible paths and trigger thumbnail load
watch(
  () => [...visiblePaths.value],
  () => {
    for (const path of visiblePaths.value) {
      const file = store.displayedFiles.find((f) => f.path === path);
      if (!file) continue;
      if (thumbnailStates.value.has(path)) continue;
      // Set loading state
      thumbnailStates.value = new Map([...thumbnailStates.value, [path, 'loading']]);
      store
        .loadThumbnail(path)
        .then(() => {
          thumbnailStates.value = new Map([...thumbnailStates.value, [path, 'loaded']]);
        })
        .catch(() => {
          thumbnailStates.value = new Map([...thumbnailStates.value, [path, 'broken']]);
        });
    }
  }
);

function getThumbSrc(path: string): string {
  return store.thumbnailCache.get(path) ?? '';
}

function handleRetry(path: string) {
  thumbnailStates.value = new Map([...thumbnailStates.value, [path, 'loading']]);
  store.retryThumbnail(path).then(() => {
    if (store.thumbnailCache.has(path)) {
      thumbnailStates.value = new Map([...thumbnailStates.value, [path, 'loaded']]);
    } else {
      thumbnailStates.value = new Map([...thumbnailStates.value, [path, 'broken']]);
    }
  });
}

function handleCardClick(index: number, event: MouseEvent) {
  // If a long-press overlay is showing, don't open lightbox
  if (longPressedPath.value) {
    longPressedPath.value = null;
    return;
  }
  const target = event.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  emit('select', index, rect);
}

// ── Keyboard navigation (roving tabindex) ──
const focusedIndex = ref(0);
const cardRefs = ref<(HTMLElement | null)[]>([]);

function getCols(): number {
  if (typeof window === 'undefined') return 3;
  const w = window.innerWidth;
  if (w >= 1024) return 4;
  if (w >= 640) return 3;
  return 2;
}

function handleGridKeydown(e: KeyboardEvent, index: number) {
  const cols = getCols();
  const len = store.displayedFiles.length;
  let next = index;

  switch (e.key) {
    case 'ArrowRight':
      next = index + 1;
      break;
    case 'ArrowLeft':
      next = index - 1;
      break;
    case 'ArrowDown':
      next = index + cols;
      break;
    case 'ArrowUp':
      next = index - cols;
      break;
    case 'Enter':
      e.preventDefault();
      emit('select', index);
      return;
    case 'Delete':
      e.preventDefault();
      triggerDeleteConfirm(store.displayedFiles[index].path);
      return;
    default:
      return;
  }

  e.preventDefault();
  // Clamp with wraparound
  if (next >= len) next = 0;
  if (next < 0) next = len - 1;
  focusedIndex.value = next;
  nextTick(() => cardRefs.value[next]?.focus());
}

// ── Long-press for touch devices (reveals action overlay) ──
const longPressedPath = ref<string | null>(null);
const isTouchDevice = computed(() => {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(hover: none)').matches;
});

const LONG_PRESS_MS = 500;
const MOVE_PX = 10;
let _lpTimers = new Map<string, ReturnType<typeof setTimeout>>();
let _lpStartX = new Map<string, number>();
let _lpStartY = new Map<string, number>();

function onLongPressDown(path: string, e: PointerEvent) {
  if (!isTouchDevice.value) return;
  _lpStartX.set(path, e.clientX);
  _lpStartY.set(path, e.clientY);
  const timer = setTimeout(() => {
    longPressedPath.value = path;
  }, LONG_PRESS_MS);
  _lpTimers.set(path, timer);
}

function onLongPressMove(path: string, e: PointerEvent) {
  const startX = _lpStartX.get(path) ?? 0;
  const startY = _lpStartY.get(path) ?? 0;
  if (Math.abs(e.clientX - startX) > MOVE_PX || Math.abs(e.clientY - startY) > MOVE_PX) {
    const timer = _lpTimers.get(path);
    if (timer) {
      clearTimeout(timer);
      _lpTimers.delete(path);
    }
  }
}

function onLongPressUp(path: string) {
  const timer = _lpTimers.get(path);
  if (timer) {
    clearTimeout(timer);
    _lpTimers.delete(path);
  }
}

function dismissOverlay() {
  longPressedPath.value = null;
}

onUnmounted(() => {
  for (const timer of _lpTimers.values()) clearTimeout(timer);
  _lpTimers.clear();
});

// ── Inline delete confirmation ──
const confirmingDelete = ref<string | null>(null);

function triggerDeleteConfirm(path: string) {
  confirmingDelete.value = path;
}

function cancelDeleteConfirm() {
  confirmingDelete.value = null;
}

async function confirmDelete(path: string) {
  try {
    await store.deleteFile(path);
  } catch {
    // deleteFile already shows error toast
  } finally {
    confirmingDelete.value = null;
  }
}

// ── Formatting helpers ──
function formatDate(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  const diffHr = Math.floor(diffMs / 3600000);

  if (diffMin < 1) return 'Just now';
  if (diffMin < 60) return `${diffMin} min ago`;
  if (diffHr < 24) return `${diffHr}h ago`;

  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  if (d.toDateString() === yesterday.toDateString()) return 'Yesterday';

  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDimensions(dims: { width: number; height: number } | null): string {
  if (!dims) return '';
  if (dims.width > 2000 || dims.height > 2000) {
    return `${(dims.width / 1000).toFixed(1)}K×${(dims.height / 1000).toFixed(1)}K`;
  }
  return `${dims.width}×${dims.height}`;
}

function aspectStyle(file: ScreenshotFile): Record<string, string> {
  if (file.dimensions) {
    return { '--aspect': `${file.dimensions.width}/${file.dimensions.height}` };
  }
  // FR-5 AC #4: default fallback is 9:16 (not 9:19.5)
  return { '--aspect': '9/16' };
}
</script>

<template>
  <div class="grid gap-3 grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
    <div
      v-for="(file, index) in store.displayedFiles"
      :key="file.path"
      :ref="
        (el) => {
          cardRefs[index] = el as HTMLElement | null;
          observeCard(el as HTMLElement | null, file.path);
        }
      "
      class="group relative rounded-lg bg-theme-card border border-theme-tertiary overflow-hidden transition-shadow duration-150 hover:shadow-lg focus-within:ring-2 focus-within:ring-accent-emerald/50"
      @click="dismissOverlay"
      @pointerdown="onLongPressDown(file.path, $event)"
      @pointermove="onLongPressMove(file.path, $event)"
      @pointerup="onLongPressUp(file.path)"
      @pointercancel="onLongPressUp(file.path)"
    >
      <!-- Card button (roving tabindex) -->
      <button
        class="w-full text-left outline-none"
        :tabindex="index === focusedIndex ? 0 : -1"
        :aria-label="`Screenshot: ${file.filename}, ${formatDate(file.created_iso)}${file.dimensions ? `, ${formatDimensions(file.dimensions)}` : ''}`"
        @click="handleCardClick(index, $event)"
        @keydown="handleGridKeydown($event, index)"
      >
        <!-- Thumbnail container -->
        <div
          class="relative aspect-[var(--aspect)] w-full bg-theme-page"
          :style="aspectStyle(file)"
        >
          <!-- Broken image placeholder with retry -->
          <div
            v-if="thumbnailStates.get(file.path) === 'broken'"
            class="w-full h-full flex flex-col items-center justify-center gap-1 text-theme-muted"
          >
            <ImageOff :size="24" />
            <span class="text-xs px-2 text-center truncate max-w-full">{{ file.filename }}</span>
            <button
              class="btn-pressable flex items-center gap-1 px-2 py-0.5 rounded text-[10px] text-accent-emerald hover:bg-accent-emerald/10 transition-colors"
              title="Retry loading image"
              @click.stop="handleRetry(file.path)"
            >
              <RotateCw :size="12" />
              Retry
            </button>
          </div>

          <!-- Loaded thumbnail -->
          <img
            v-else-if="thumbnailStates.get(file.path) === 'loaded' && getThumbSrc(file.path)"
            :src="getThumbSrc(file.path)"
            :alt="file.filename"
            loading="lazy"
            class="w-full h-full object-cover"
            @error="thumbnailStates = new Map([...thumbnailStates, [file.path, 'broken']])"
          />

          <!-- Loading spinner -->
          <div
            v-else-if="thumbnailStates.get(file.path) === 'loading'"
            class="w-full h-full flex items-center justify-center bg-theme-card"
          >
            <Loader2 :size="20" class="animate-spin text-theme-muted" />
          </div>

          <!-- Skeleton placeholder (not yet visible) -->
          <div v-else class="w-full h-full bg-theme-card animate-pulse" />

          <!-- Dimensions badge -->
          <span
            v-if="file.dimensions"
            class="absolute top-1.5 right-1.5 px-1.5 py-0.5 rounded text-[10px] font-mono bg-black/50 text-white/80"
          >
            {{ formatDimensions(file.dimensions) }}
          </span>
        </div>

        <!-- Info bar below thumbnail -->
        <div class="px-2 py-1.5">
          <p class="text-xs text-theme-secondary truncate">{{ formatDate(file.created_iso) }}</p>
          <p class="text-[10px] text-theme-muted">{{ formatSize(file.size_bytes) }}</p>
        </div>
      </button>

      <!-- Hover/touch action overlay -->
      <div
        class="absolute inset-x-0 bottom-0 flex items-center justify-center gap-1 px-2 py-2 bg-gradient-to-t from-black/70 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-150"
        :class="{ 'opacity-100': confirmingDelete === file.path || longPressedPath === file.path }"
        @click.stop
      >
        <template v-if="confirmingDelete === file.path">
          <!-- Inline delete confirmation -->
          <span class="text-xs text-white mr-1">Delete?</span>
          <button
            class="btn-pressable p-1 rounded text-white hover:bg-white/20"
            title="Confirm delete"
            @click.stop="confirmDelete(file.path)"
          >
            ✓
          </button>
          <button
            class="btn-pressable p-1 rounded text-white hover:bg-white/20"
            title="Cancel"
            @click.stop="cancelDeleteConfirm"
          >
            ✗
          </button>
        </template>
        <template v-else>
          <button
            class="btn-pressable p-1.5 rounded text-white/80 hover:text-white hover:bg-white/15"
            title="Open folder"
            @click.stop="store.openFolder(file.path)"
          >
            <FolderOpen :size="16" />
          </button>
          <button
            class="btn-pressable p-1.5 rounded text-white/80 hover:text-white hover:bg-white/15"
            title="Open in OS viewer"
            @click.stop="store.openFile(file.path)"
          >
            <Monitor :size="16" />
          </button>
          <button
            class="btn-pressable p-1.5 rounded text-white/80 hover:text-white hover:bg-white/15"
            title="Copy path"
            @click.stop="store.copyPath(file.path)"
          >
            <Copy :size="16" />
          </button>
          <button
            class="btn-pressable p-1.5 rounded text-red-400 hover:text-red-300 hover:bg-red-500/15"
            title="Delete screenshot"
            @click.stop="triggerDeleteConfirm(file.path)"
          >
            <Trash2 :size="16" />
          </button>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Long-press overlay handled via JS state (longPressedPath) */
</style>
