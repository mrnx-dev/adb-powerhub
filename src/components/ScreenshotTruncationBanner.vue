<script setup lang="ts">
defineProps<{
  totalCount: number;
  shownCount: number;
}>();

defineEmits<{
  dismiss: [];
}>();
</script>

<template>
  <div
    role="status"
    aria-live="polite"
    class="flex items-center gap-3 px-4 py-2.5 rounded-lg border border-accent-emerald/20 bg-accent-emerald/5 text-sm"
  >
    <span class="text-accent-emerald flex-shrink-0" aria-hidden="true">ℹ️</span>
    <span class="text-theme-secondary flex-1">
      Showing the
      <strong class="text-theme-primary">{{ shownCount }}</strong> most recent screenshots.
      <strong class="text-theme-primary">{{ totalCount }}</strong> total files in folder.
    </span>
    <button
      class="btn-pressable p-1 rounded text-theme-muted hover:text-theme-primary transition-colors"
      aria-label="Dismiss truncation banner"
      @click="$emit('dismiss')"
    >
      ✕
    </button>
  </div>
</template>
