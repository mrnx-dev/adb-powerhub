<script setup lang="ts">
import type { DependencyStatus } from '../../types/about';

defineProps<{
  dependencies: DependencyStatus[];
}>();

function badgeClass(available: boolean) {
  return available ? 'status-badge status-badge-active' : 'status-badge status-badge-error';
}

function dotClass(available: boolean) {
  return available ? 'status-dot status-dot-active' : 'status-dot status-dot-error';
}

function versionLabel(dep: DependencyStatus) {
  return dep.available && dep.version ? dep.version : 'Missing';
}
</script>

<template>
  <section class="about-section" aria-labelledby="deps-label">
    <h2 id="deps-label" class="section-label">Dependencies</h2>
    <div v-if="dependencies.length === 0" class="text-sm text-theme-muted">
      No dependencies configured.
    </div>
    <div v-else class="dep-list flex flex-col gap-3">
      <div
        v-for="dep in dependencies"
        :key="dep.name"
        class="dep-row hover-subtle flex items-center justify-between gap-3 p-3 rounded-md bg-theme-btn border border-theme-tertiary transition-colors duration-200 ease-out"
      >
        <div class="dep-info min-w-0">
          <div class="dep-name text-sm font-semibold text-theme-primary">{{ dep.name }}</div>
          <div class="dep-detail text-xs text-theme-muted mt-0.5 font-mono break-all">
            {{ dep.path || '—' }}
          </div>
        </div>
        <span :class="badgeClass(dep.available)">
          <span :class="dotClass(dep.available)"></span>
          {{ versionLabel(dep) }}
        </span>
      </div>
    </div>
  </section>
</template>
