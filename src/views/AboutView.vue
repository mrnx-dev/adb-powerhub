<script setup lang="ts">
import { onMounted } from 'vue';
import { useAboutStore } from '../stores/about';
import AppIdentityHeader from '../components/about/AppIdentityHeader.vue';
import DependencyList from '../components/about/DependencyList.vue';
import ExternalLinks from '../components/about/ExternalLinks.vue';
import LicenseNotice from '../components/about/LicenseNotice.vue';
import SupportActions from '../components/about/SupportActions.vue';

const aboutStore = useAboutStore();

onMounted(() => {
  aboutStore.load();
});
</script>

<template>
  <div tabindex="-1" class="flex-1 min-h-0 overflow-y-auto bg-glow p-6">
    <div class="about-grid grid grid-cols-1 lg:grid-cols-2 gap-6">
      <section class="card-glass p-4 about-section">
        <AppIdentityHeader :app-info="aboutStore.appInfo" />
      </section>

      <section class="card-glass p-4 about-section">
        <DependencyList :dependencies="aboutStore.dependencies" />
      </section>

      <section class="card-glass p-4 about-section lg:col-span-2">
        <ExternalLinks />
        <LicenseNotice class="mt-6" />
        <SupportActions class="mt-6" />
      </section>
    </div>
  </div>
</template>

<style scoped>
.about-section {
  opacity: 0;
  transform: translateY(12px) scale(0.98);
  animation: about-section-enter var(--duration-card) var(--ease-out) forwards;
}
.about-section:nth-child(1) {
  animation-delay: 0ms;
}
.about-section:nth-child(2) {
  animation-delay: 60ms;
}
.about-section:nth-child(3) {
  animation-delay: 120ms;
}

@keyframes about-section-enter {
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .about-section {
    animation: none;
    opacity: 1;
    transform: none;
  }
}
</style>
