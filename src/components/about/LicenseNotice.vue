<script setup lang="ts">
const currentYear = new Date().getFullYear();
</script>

<template>
  <section class="about-section" aria-labelledby="legal-label">
    <h2 id="legal-label" class="section-label">License & Privacy</h2>
    <p class="legal-text text-sm text-theme-secondary leading-relaxed m-0">
      <strong class="text-theme-primary">ADB PowerHub</strong> is proprietary software. All rights
      reserved. © {{ currentYear }}. No data is collected; all ADB commands run locally.
    </p>
  </section>
</template>
