<script setup lang="ts">
import { computed } from 'vue';
import type { AppInfo } from '../../types/about';

const props = defineProps<{
  appInfo: AppInfo | null;
}>();

const version = computed(() => props.appInfo?.version ?? '');
const commit = computed(() => props.appInfo?.commit ?? '');
const environment = computed(() => props.appInfo?.environment ?? 'development');
</script>

<template>
  <div class="about-header flex items-center gap-4">
    <div
      class="about-icon w-16 h-16 rounded-xl bg-color-primary-container border border-theme-primary flex items-center justify-center text-color-primary text-[28px] font-extrabold shrink-0"
      aria-hidden="true"
    >
      P
    </div>
    <div class="min-w-0">
      <h1 class="text-xl font-extrabold text-theme-heading m-0">ADB PowerHub</h1>
      <p class="text-sm text-theme-secondary mt-1">
        Control your Android devices from the desktop.
      </p>
      <div class="about-meta flex items-center gap-2 mt-2 flex-wrap">
        <span v-if="version" class="status-badge status-badge-active">
          <span class="status-dot status-dot-active"></span>
          v{{ version }}
        </span>
        <span v-if="commit" class="status-badge status-badge-idle">
          {{ commit }}
        </span>
        <span v-if="environment === 'development'" class="status-badge status-badge-warning">
          <span class="status-dot status-dot-warning"></span>
          Development
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.about-icon {
  transition: transform var(--duration-quick) var(--ease-out);
}

@media (hover: hover) and (pointer: fine) {
  .about-header:hover .about-icon {
    transform: scale(1.02);
  }
}

@media (prefers-reduced-motion: reduce) {
  .about-icon {
    transition: none;
  }
}
</style>
