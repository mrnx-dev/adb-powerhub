<script setup lang="ts">
import { onMounted } from 'vue';
import { useAboutStore } from '../stores/about';
import AboutCard from '../components/about/AboutCard.vue';

const aboutStore = useAboutStore();

onMounted(() => {
  aboutStore.load();
});
</script>

<template>
  <div tabindex="-1" class="flex-1 min-h-0 overflow-y-auto bg-glow p-6">
    <div class="max-w-[640px] mx-auto">
      <AboutCard :app-info="aboutStore.appInfo" :dependencies="aboutStore.dependencies" />
    </div>
  </div>
</template>
