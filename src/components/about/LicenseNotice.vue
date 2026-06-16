<script setup lang="ts">
const currentYear = new Date().getFullYear();
</script>

<template>
  <div aria-labelledby="legal-label">
    <h2 id="legal-label" class="font-sans text-xs font-semibold tracking-wider uppercase mb-4">
      License & Privacy
    </h2>
    <p class="legal-text text-sm text-theme-secondary leading-relaxed m-0">
      <strong class="text-theme-primary">ADB PowerHub</strong> is proprietary software. All rights
      reserved. © {{ currentYear }}. No data is collected; all ADB commands run locally.
    </p>
  </div>
</template>
