<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue';
import {
  X,
  Check,
  ChevronLeft,
  ChevronRight,
  FolderOpen,
  Monitor,
  Copy,
  Trash2,
  ImageOff,
  Loader2,
  Plus,
  Minus,
  Maximize,
  Minimize,
  ClipboardCopy,
} from '@lucide/vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useScreenshotsStore } from '../stores/screenshots';
import { useToastStore } from '../stores/toast';

const store = useScreenshotsStore();
const toast = useToastStore();

const lightboxRef = ref<HTMLElement | null>(null);
const imageEl = ref<HTMLImageElement | null>(null);
const imageLoading = ref(true);
const imageError = ref(false);
const confirmingDelete = ref(false);
let previousFocus: HTMLElement | null = null;

// ── Zoom + Pan state ──
const zoomScale = ref(1);
const zoomTx = ref(0);
const zoomTy = ref(0);
const isPanning = ref(false);
const panStartX = ref(0);
const panStartY = ref(0);
const zoomContainerRef = ref<HTMLDivElement | null>(null);
const flipAnimating = ref(false);

const MIN_SCALE = 0.5;
const MAX_SCALE = 5;
const ZOOM_STEP = 0.25;

const zoomStyle = computed(() => ({
  transform: `translate(${zoomTx.value}px, ${zoomTy.value}px) scale(${zoomScale.value})`,
  transition: isPanning.value ? 'none' : 'transform 200ms cubic-bezier(0.23, 1, 0.32, 1)',
}));

const zoomPercentLabel = computed(() => {
  if (zoomScale.value === 1) return 'Fit';
  return `${Math.round(zoomScale.value * 100)}%`;
});

function resetZoom() {
  zoomScale.value = 1;
  zoomTx.value = 0;
  zoomTy.value = 0;
}

function smoothZoom(delta: number, clientX: number, clientY: number) {
  if (flipAnimating.value) return;
  const container = zoomContainerRef.value;
  if (!container) return;

  const rect = container.getBoundingClientRect();
  const cx = clientX - rect.left;
  const cy = clientY - rect.top;
  const centerX = rect.width / 2;
  const centerY = rect.height / 2;

  const prevScale = zoomScale.value;
  const newScale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, prevScale + delta));
  if (newScale === prevScale) return;

  // Zoom toward cursor using viewport-center origin (matches preview math).
  // Image-local coordinates of the point under cursor, relative to image center.
  const imgLocalX = (cx - centerX - zoomTx.value) / prevScale;
  const imgLocalY = (cy - centerY - zoomTy.value) / prevScale;

  // After scale change, keep the same image-local point under the cursor.
  zoomTx.value = cx - centerX - newScale * imgLocalX;
  zoomTy.value = cy - centerY - newScale * imgLocalY;
  zoomScale.value = newScale;
}

function zoomIn() {
  smoothZoom(ZOOM_STEP, window.innerWidth / 2, window.innerHeight / 2);
}

function zoomOut() {
  smoothZoom(-ZOOM_STEP, window.innerWidth / 2, window.innerHeight / 2);
}

function onWheel(e: WheelEvent) {
  if (!store.lightboxOpen) return;
  e.preventDefault();
  const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
  smoothZoom(delta, e.clientX, e.clientY);
  hideHint();
}

function onDblClick(e: MouseEvent) {
  if (!store.lightboxOpen || flipAnimating.value) return;
  if (zoomScale.value !== 1 || zoomTx.value !== 0 || zoomTy.value !== 0) {
    resetZoom();
  } else {
    smoothZoom(2, e.clientX, e.clientY);
  }
  hideHint();
}

function onPointerDown(e: PointerEvent) {
  if (!store.lightboxOpen || flipAnimating.value) return;
  isPanning.value = true;
  panStartX.value = e.clientX - zoomTx.value;
  panStartY.value = e.clientY - zoomTy.value;
  (e.currentTarget as HTMLElement)?.setPointerCapture?.(e.pointerId);
  hideHint();
}

function onPointerMove(e: PointerEvent) {
  if (!isPanning.value) return;
  zoomTx.value = e.clientX - panStartX.value;
  zoomTy.value = e.clientY - panStartY.value;
}

function onPointerUp() {
  isPanning.value = false;
}

// ── Fullscreen ──
const isFullscreen = ref(false);

async function toggleFullscreen() {
  try {
    const win = getCurrentWindow();
    if (isFullscreen.value) {
      await win.setFullscreen(false);
      isFullscreen.value = false;
    } else {
      await win.setFullscreen(true);
      isFullscreen.value = true;
    }
  } catch {
    // Fullscreen not supported or failed
  }
}

// ── Copy image ──
async function copyImageToClipboard() {
  if (!lightboxSrc.value) return;
  try {
    const res = await fetch(lightboxSrc.value);
    const blob = await res.blob();
    await navigator.clipboard.write([new ClipboardItem({ [blob.type]: blob })]);
    toast.show('Image copied to clipboard', 'success');
  } catch {
    toast.show('Failed to copy image', 'error');
  }
}

// ── Hints ──
const hintVisible = ref(true);

function hideHint() {
  hintVisible.value = false;
}

const current = computed(() => store.currentLightboxFile);
const lightboxSrc = ref('');

// ── FLIP transition state ──
const prefersReducedMotion = ref(false);
let reducedMotionQuery: MediaQueryList | null = null;

onMounted(() => {
  if (typeof window !== 'undefined') {
    reducedMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    prefersReducedMotion.value = reducedMotionQuery.matches;
    const handler = (e: MediaQueryListEvent) => {
      prefersReducedMotion.value = e.matches;
    };
    reducedMotionQuery.addEventListener('change', handler);
    window.addEventListener('pointermove', onPointerMove);
    window.addEventListener('pointerup', onPointerUp);
  }
});

onBeforeUnmount(() => {
  if (reducedMotionQuery) {
    reducedMotionQuery.removeEventListener('change', () => {});
  }
  window.removeEventListener('pointermove', onPointerMove);
  window.removeEventListener('pointerup', onPointerUp);
});

// ── Image loading via store unified cache ──
async function loadCurrentImage() {
  const path = current.value?.path;
  if (!path) return;

  imageLoading.value = true;
  imageError.value = false;

  try {
    const url = await store.loadImage(path);
    lightboxSrc.value = url;
    imageLoading.value = false;

    // Apply FLIP transition after image loads
    nextTick(() => {
      if (!imageEl.value) return;
      const sourceRect = store.lightboxSourceRect;

      if (!sourceRect || prefersReducedMotion.value) {
        return;
      }

      const destRect = imageEl.value.getBoundingClientRect();
      const dx = sourceRect.left - destRect.left;
      const dy = sourceRect.top - destRect.top;
      const scaleX = sourceRect.width / destRect.width;
      const scaleY = sourceRect.height / destRect.height;

      if (dx === 0 && dy === 0 && scaleX === 1 && scaleY === 1) return;

      imageEl.value.style.transformOrigin = 'top left';
      flipAnimating.value = true;
      imageEl.value.style.transform = `translate(${dx}px, ${dy}px) scale(${scaleX}, ${scaleY})`;
      imageEl.value.style.opacity = '0.8';

      void imageEl.value.offsetHeight;

      const duration = prefersReducedMotion.value ? '0ms' : '250ms';
      imageEl.value.style.transition = `transform ${duration} ease-out, opacity 150ms ease-out`;
      imageEl.value.style.transform = 'translate(0, 0) scale(1, 1)';
      imageEl.value.style.opacity = '1';

      setTimeout(() => {
        flipAnimating.value = false;
        if (imageEl.value) {
          imageEl.value.style.transition = '';
          imageEl.value.style.transformOrigin = '';
        }
      }, 260);
    });
  } catch {
    imageError.value = true;
    imageLoading.value = false;
  }
}

watch(
  () => store.lightboxOpen,
  (open) => {
    if (open) {
      resetZoom();
      hintVisible.value = true;
      previousFocus = document.activeElement as HTMLElement | null;
      loadCurrentImage();
      nextTick(() => {
        const closeBtn = lightboxRef.value?.querySelector('[data-close]') as HTMLElement | null;
        if (closeBtn) closeBtn.focus();
      });
    } else {
      previousFocus?.focus();
      previousFocus = null;
      confirmingDelete.value = false;
      lightboxSrc.value = '';
      resetZoom();
      hintVisible.value = true;
      if (isFullscreen.value) {
        getCurrentWindow()
          .setFullscreen(false)
          .catch(() => {});
        isFullscreen.value = false;
      }
    }
  }
);

// Reload when navigating prev/next
watch(
  () => store.lightboxIndex,
  () => {
    if (store.lightboxOpen) {
      resetZoom();
      if (lightboxSrc.value && !prefersReducedMotion.value) {
        lightboxSrc.value = '';
        nextTick(() => {
          loadCurrentImage();
        });
      } else {
        loadCurrentImage();
      }
    }
  }
);

// ── Close animation (FLIP zoom-out) ──
function closeWithAnimation() {
  resetZoom();
  if (!imageEl.value || prefersReducedMotion.value) {
    store.closeLightbox();
    return;
  }

  const sourceRect = store.lightboxSourceRect;
  if (!sourceRect) {
    store.closeLightbox();
    return;
  }

  const destRect = imageEl.value.getBoundingClientRect();
  const dx = sourceRect.left - destRect.left;
  const dy = sourceRect.top - destRect.top;
  const scaleX = sourceRect.width / destRect.width;
  const scaleY = sourceRect.height / destRect.height;

  imageEl.value.style.transition = 'transform 200ms ease-in, opacity 150ms ease-in';
  imageEl.value.style.transformOrigin = 'top left';
  imageEl.value.style.transform = `translate(${dx}px, ${dy}px) scale(${scaleX}, ${scaleY})`;
  imageEl.value.style.opacity = '0';

  setTimeout(() => {
    store.closeLightbox();
  }, 200);
}

// ── Keyboard handler ──
function handleKeydown(e: KeyboardEvent) {
  switch (e.key) {
    case 'Escape':
      if (confirmingDelete.value) {
        confirmingDelete.value = false;
      } else {
        closeWithAnimation();
      }
      break;
    case 'ArrowLeft':
      e.preventDefault();
      store.prevImage();
      break;
    case 'ArrowRight':
      e.preventDefault();
      store.nextImage();
      break;
    case 'Delete':
      if (!confirmingDelete.value) {
        triggerDelete();
      }
      break;
    case 'o':
    case 'O':
      if (!e.ctrlKey && current.value) {
        store.openFile(current.value.path);
      }
      break;
    case 'f':
    case 'F':
      if (!e.ctrlKey) {
        e.preventDefault();
        toggleFullscreen();
      }
      break;
    case 'c':
    case 'C':
      if (!e.ctrlKey) {
        e.preventDefault();
        copyImageToClipboard();
      }
      break;
    case '+':
    case '=':
      e.preventDefault();
      zoomIn();
      break;
    case '-':
    case '_':
      e.preventDefault();
      zoomOut();
      break;
    case '0':
      e.preventDefault();
      resetZoom();
      break;
    case 'Tab':
      trapFocus(e);
      break;
  }
}

// ── Focus trap ──
function trapFocus(e: KeyboardEvent) {
  if (!lightboxRef.value) return;
  const FOCUSABLE = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';
  const focusable = Array.from(lightboxRef.value.querySelectorAll(FOCUSABLE)) as HTMLElement[];
  if (focusable.length === 0) return;

  const first = focusable[0];
  const last = focusable[focusable.length - 1];

  if (e.shiftKey && document.activeElement === first) {
    e.preventDefault();
    last.focus();
  } else if (!e.shiftKey && document.activeElement === last) {
    e.preventDefault();
    first.focus();
  }
}

// ── Delete from lightbox ──
function triggerDelete() {
  confirmingDelete.value = true;
}

function cancelDelete() {
  confirmingDelete.value = false;
}

async function confirmDelete() {
  if (!current.value) return;
  try {
    await store.deleteFile(current.value.path);
  } catch {
    // Error toast already shown by store
  } finally {
    confirmingDelete.value = false;
  }
}

// ── Helpers ──
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
}
</script>

<template>
  <Teleport to="body">
    <Transition name="lightbox">
      <div
        v-if="store.lightboxOpen && current"
        ref="lightboxRef"
        class="fixed inset-0 z-[100] flex flex-col"
        role="dialog"
        aria-modal="true"
        aria-label="Image preview"
        @keydown="handleKeydown"
      >
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/85 backdrop-blur-sm" @click="closeWithAnimation()" />

        <!-- Top centered toolbar -->
        <div
          class="absolute top-4 left-1/2 -translate-x-1/2 z-[110] flex items-center gap-0.5 px-2 py-1.5 rounded-xl bg-white/[0.08] backdrop-blur-md border border-white/10 shadow-lg"
          role="toolbar"
          aria-label="Lightbox controls"
        >
          <button class="lb-btn btn-pressable" title="Previous (←)" @click="store.prevImage()">
            <ChevronLeft :size="18" />
          </button>
          <button class="lb-btn btn-pressable" title="Zoom out (-)" @click="zoomOut()">
            <Minus :size="18" />
          </button>
          <button class="lb-btn btn-pressable" title="Fit to screen (0)" @click="resetZoom()">
            <Maximize :size="18" />
          </button>
          <button class="lb-btn btn-pressable" title="Zoom in (+)" @click="zoomIn()">
            <Plus :size="18" />
          </button>

          <div class="w-px h-5 bg-white/10 mx-1" />

          <button
            class="lb-btn btn-pressable"
            title="Toggle fullscreen (F)"
            @click="toggleFullscreen()"
          >
            <component :is="isFullscreen ? Minimize : Maximize" :size="18" />
          </button>
          <button
            class="lb-btn btn-pressable"
            title="Copy image (C)"
            @click="copyImageToClipboard()"
          >
            <ClipboardCopy :size="18" />
          </button>

          <div class="w-px h-5 bg-white/10 mx-1" />

          <button
            class="lb-btn btn-pressable"
            title="Close (Esc)"
            data-close
            @click="closeWithAnimation()"
          >
            <X :size="18" />
          </button>
          <button class="lb-btn btn-pressable" title="Next (→)" @click="store.nextImage()">
            <ChevronRight :size="18" />
          </button>
        </div>

        <!-- Nav arrows -->
        <template v-if="store.displayedFiles.length > 1">
          <button
            class="lb-nav-arrow btn-pressable left-4"
            title="Previous"
            @click="store.prevImage()"
          >
            <ChevronLeft :size="22" />
          </button>
          <button
            class="lb-nav-arrow btn-pressable right-4"
            title="Next"
            @click="store.nextImage()"
          >
            <ChevronRight :size="22" />
          </button>
        </template>

        <!-- Image viewport -->
        <div
          ref="zoomContainerRef"
          class="relative z-10 flex-1 flex items-center justify-center min-h-0 overflow-hidden"
          :class="{ 'cursor-grab': !isPanning && zoomScale > 1, 'cursor-grabbing': isPanning }"
          @wheel="onWheel"
          @dblclick="onDblClick"
          @pointerdown="onPointerDown"
        >
          <!-- Loading spinner -->
          <Loader2
            v-if="imageLoading && !imageError"
            :size="32"
            class="animate-spin text-white/60 absolute z-10"
          />

          <!-- Zoomed image wrapper -->
          <div v-if="lightboxSrc && !imageError" :style="zoomStyle" class="will-change-transform">
            <img
              ref="imageEl"
              :key="current.path"
              :src="lightboxSrc"
              :alt="current.filename"
              class="max-w-[90vw] max-h-[80vh] object-contain rounded shadow-2xl select-none"
              draggable="false"
              @load="imageLoading = false"
              @error="
                imageError = true;
                imageLoading = false;
              "
            />
          </div>

          <!-- Error fallback -->
          <div v-if="imageError" class="text-white/60 text-center flex flex-col items-center gap-3">
            <ImageOff :size="48" />
            <p>Image could not be loaded</p>
            <button
              class="btn-pressable px-4 py-2 rounded-lg bg-white/10 text-white hover:bg-white/20 text-sm"
              @click="store.openFile(current.path)"
            >
              Open in OS Viewer
            </button>
          </div>
        </div>

        <!-- Info bar (centered bottom, above metadata) -->
        <div
          class="absolute bottom-[3.25rem] left-1/2 -translate-x-1/2 z-[110] flex items-center gap-3 px-3 py-1.5 rounded-lg bg-white/[0.08] backdrop-blur-md border border-white/10 text-xs text-white/60"
        >
          <span>{{ store.lightboxIndex + 1 }} / {{ store.displayedFiles.length }}</span>
          <span v-if="current.dimensions" class="w-px h-3 bg-white/10" />
          <span v-if="current.dimensions"
            >{{ current.dimensions.width }}×{{ current.dimensions.height }}</span
          >
          <span class="w-px h-3 bg-white/10" />
          <span>{{ formatSize(current.size_bytes) }}</span>
          <span class="w-px h-3 bg-white/10" />
          <span class="text-white/90 font-medium">{{ zoomPercentLabel }}</span>
        </div>

        <!-- Bottom metadata + action bar -->
        <div
          class="relative z-10 flex items-center justify-between px-4 py-3 border-t border-white/10 flex-wrap gap-y-2"
        >
          <div class="text-white/70 text-xs flex flex-wrap gap-x-3 gap-y-1">
            <span class="truncate max-w-[180px]">{{ current.filename }}</span>
            <span>{{ formatDate(current.created_iso) }}</span>
          </div>

          <div class="flex items-center gap-1">
            <template v-if="confirmingDelete">
              <span class="text-xs text-white/80 mr-1">Delete?</span>
              <button
                class="btn-pressable p-1.5 rounded text-white hover:bg-white/20"
                @click="confirmDelete"
              >
                <Check :size="16" />
              </button>
              <button
                class="btn-pressable p-1.5 rounded text-white hover:bg-white/20"
                @click="cancelDelete"
              >
                <X :size="16" />
              </button>
            </template>
            <template v-else>
              <button
                class="btn-pressable p-1.5 rounded text-white/70 hover:text-white hover:bg-white/10"
                title="Open folder"
                @click="store.openFolder(current.path)"
              >
                <FolderOpen :size="18" />
              </button>
              <button
                class="btn-pressable p-1.5 rounded text-white/70 hover:text-white hover:bg-white/10"
                title="Open in OS viewer"
                @click="store.openFile(current.path)"
              >
                <Monitor :size="18" />
              </button>
              <button
                class="btn-pressable p-1.5 rounded text-white/70 hover:text-white hover:bg-white/10"
                title="Copy path"
                @click="store.copyPath(current.path)"
              >
                <Copy :size="18" />
              </button>
              <button
                class="btn-pressable p-1.5 rounded text-red-400 hover:text-red-300 hover:bg-red-500/15"
                title="Delete screenshot"
                @click="triggerDelete"
              >
                <Trash2 :size="18" />
              </button>
            </template>
          </div>
        </div>

        <!-- Interaction hints -->
        <div
          class="absolute bottom-[6.5rem] left-1/2 -translate-x-1/2 z-[105] text-center text-xs text-white/40 pointer-events-none transition-opacity duration-300"
          :class="{ 'opacity-0': !hintVisible }"
        >
          <p>Double-click to zoom • Scroll to zoom • Drag to pan</p>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* Entry/exit with scale(0.95) + opacity per Emil */
.lightbox-enter-active {
  transition:
    opacity 250ms cubic-bezier(0.23, 1, 0.32, 1),
    transform 250ms cubic-bezier(0.23, 1, 0.32, 1);
}
.lightbox-leave-active {
  transition:
    opacity 200ms cubic-bezier(0.23, 1, 0.32, 1),
    transform 200ms cubic-bezier(0.23, 1, 0.32, 1);
}
.lightbox-enter-from {
  opacity: 0;
  transform: scale(0.95);
}
.lightbox-leave-to {
  opacity: 0;
  transform: scale(0.98);
}

@media (prefers-reduced-motion: reduce) {
  .lightbox-enter-active,
  .lightbox-leave-active {
    transition: opacity 100ms ease;
    transform: none !important;
  }
  .lightbox-enter-from,
  .lightbox-leave-to {
    transform: none !important;
  }
}

/* Toolbar button */
.lb-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 0.5rem;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.55);
  cursor: pointer;
  transition:
    transform var(--duration-quick) var(--ease-out),
    background-color var(--duration-quick) var(--ease-out),
    color var(--duration-quick) var(--ease-out);
}

@media (hover: hover) and (pointer: fine) {
  .lb-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: rgba(255, 255, 255, 0.9);
  }
}

.lb-btn:active {
  transform: scale(0.93);
}

/* Nav arrows */
.lb-nav-arrow {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  z-index: 110;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  color: rgba(255, 255, 255, 0.55);
  cursor: pointer;
  transition:
    transform var(--duration-quick) var(--ease-out),
    background-color var(--duration-quick) var(--ease-out),
    color var(--duration-quick) var(--ease-out);
}

@media (hover: hover) and (pointer: fine) {
  .lb-nav-arrow:hover {
    background: rgba(255, 255, 255, 0.14);
    color: rgba(255, 255, 255, 0.9);
  }
}

.lb-nav-arrow:active {
  transform: translateY(-50%) scale(0.93);
}
</style>
