<script setup lang="ts">
import { ref, nextTick, watch, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { FolderOpen, Monitor, Copy, Trash2, ImageOff, Loader2 } from '@lucide/vue';
import { useScreenshotsStore, type ScreenshotFile } from '../stores/screenshots';

const store = useScreenshotsStore();

const emit = defineEmits<{
  select: [index: number];
}>();

// ── Lazy loading image visibility ──
const imageVisible = ref<Set<number>>(new Set());
const cardObserver = ref<IntersectionObserver | null>(null);

function initObserver() {
  cardObserver.value = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          const index = Number((entry.target as HTMLElement).dataset.index);
          if (!isNaN(index)) {
            imageVisible.value = new Set([...imageVisible.value, index]);
          }
          cardObserver.value?.unobserve(entry.target);
        }
      }
    },
    { rootMargin: '200%' }
  );
}

function observeCard(el: HTMLElement | null, index: number) {
  if (el && cardObserver.value) {
    el.dataset.index = String(index);
    cardObserver.value.observe(el);
  }
}

// Re-initialize observer when displayed files change
onMounted(() => initObserver());
watch(
  () => store.displayedFiles,
  () => {
    nextTick(() => {
      // Reset visibility on list change
      imageVisible.value = new Set();
      if (cardObserver.value) cardObserver.value.disconnect();
      initObserver();
    });
  }
);
onBeforeUnmount(() => cardObserver.value?.disconnect());

// ── Image loading state ──
const brokenImages = ref<Set<number>>(new Set());
const loadedUrls = ref<Map<number, string>>(new Map());
const loadingIndices = ref<Set<number>>(new Set());

// Watch for newly visible cards and trigger base64 load
watch(
  () => [...imageVisible.value],
  (visible) => {
    for (const index of visible) {
      const file = store.displayedFiles[index];
      if (!file || loadedUrls.value.has(index) || loadingIndices.value.has(index)) continue;
      loadingIndices.value = new Set([...loadingIndices.value, index]);
      loadImageBase64(file.path)
        .then((url) => {
          loadedUrls.value = new Map([...loadedUrls.value, [index, url]]);
          loadingIndices.value = new Set([...loadingIndices.value].filter((i) => i !== index));
        })
        .catch(() => {
          brokenImages.value = new Set([...brokenImages.value, index]);
          loadingIndices.value = new Set([...loadingIndices.value].filter((i) => i !== index));
        });
    }
  }
);

function onImageError(index: number) {
  brokenImages.value = new Set([...brokenImages.value, index]);
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
// ── Base64 image cache ──
const imageCache = new Map<string, string>();
const loadingImages = new Set<string>();

async function loadImageBase64(path: string): Promise<string> {
  if (imageCache.has(path)) return imageCache.get(path)!;
  if (loadingImages.has(path)) {
    // Wait for in-flight load
    return new Promise((resolve) => {
      const check = setInterval(() => {
        if (imageCache.has(path)) {
          clearInterval(check);
          resolve(imageCache.get(path)!);
        }
      }, 50);
    });
  }
  loadingImages.add(path);
  try {
    const base64 = await invoke<string>('adb_read_file_base64', { path });
    const url = `data:image/png;base64,${base64}`;
    imageCache.set(path, url);
    return url;
  } finally {
    loadingImages.delete(path);
  }
}

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
  return { '--aspect': '9/19.5' };
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
          observeCard(el as HTMLElement | null, index);
        }
      "
      class="group relative rounded-lg bg-theme-card border border-theme-tertiary overflow-hidden transition-shadow duration-150 hover:shadow-lg focus-within:ring-2 focus-within:ring-accent-emerald/50"
    >
      <!-- Card button (roving tabindex) -->
      <button
        class="w-full text-left outline-none"
        :tabindex="index === focusedIndex ? 0 : -1"
        :aria-label="`Screenshot: ${file.filename}, ${formatDate(file.created_iso)}${file.dimensions ? `, ${formatDimensions(file.dimensions)}` : ''}`"
        @click="emit('select', index)"
        @keydown="handleGridKeydown($event, index)"
      >
        <!-- Thumbnail container -->
        <div
          class="relative aspect-[var(--aspect)] w-full bg-theme-page"
          :style="aspectStyle(file)"
        >
          <!-- Broken image placeholder -->
          <div
            v-if="brokenImages.has(index)"
            class="w-full h-full flex flex-col items-center justify-center gap-1 text-theme-muted"
          >
            <ImageOff :size="24" />
            <span class="text-xs px-2 text-center truncate max-w-full">{{ file.filename }}</span>
          </div>

          <!-- Lazy-loaded image -->
          <img
            v-else-if="loadedUrls.has(index)"
            :src="loadedUrls.get(index)"
            :alt="file.filename"
            loading="lazy"
            class="w-full h-full object-cover"
            @error="onImageError(index)"
          />

          <!-- Loading spinner while base64 loads -->
          <div
            v-else-if="imageVisible.has(index) && loadingIndices.has(index)"
            class="w-full h-full flex items-center justify-center bg-theme-card"
          >
            <Loader2 :size="20" class="animate-spin text-theme-muted" />
          </div>

          <!-- Skeleton placeholder -->
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

      <!-- Hover action overlay (desktop only) -->
      <div
        class="absolute inset-x-0 bottom-0 flex items-center justify-center gap-1 px-2 py-2 bg-gradient-to-t from-black/70 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-150"
        :class="{ 'opacity-100': confirmingDelete === file.path }"
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
@media (hover: none) {
  .group:hover .opacity-0.group-hover\:opacity-100 {
    opacity: 1;
  }
}
</style>
