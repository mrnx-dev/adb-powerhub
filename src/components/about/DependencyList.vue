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
  <div aria-labelledby="deps-label">
    <h2 id="deps-label" class="font-sans text-xs font-semibold tracking-wider uppercase mb-4">
      Dependencies
    </h2>
    <div v-if="dependencies.length === 0" class="text-sm text-theme-muted">
      No dependencies configured.
    </div>
    <div v-else class="dep-list flex flex-col gap-3">
      <div
        v-for="(dep, index) in dependencies"
        :key="dep.name"
        :style="{ '--stagger-index': index }"
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
  </div>
</template>

<style scoped>
.dep-row {
  opacity: 0;
  transform: translateY(8px);
  animation: dep-row-enter 180ms var(--ease-out) forwards;
  animation-delay: calc(var(--stagger-index, 0) * 50ms);
}

@keyframes dep-row-enter {
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .dep-row {
    animation: none;
    opacity: 1;
    transform: none;
  }
}
</style>
